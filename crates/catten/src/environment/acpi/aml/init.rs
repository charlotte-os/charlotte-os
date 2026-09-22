//! ACPI interpreter startup. Call from a kernel thread after timers, the heap,
//! and interrupt controllers are available.

use core::ffi::CStr;
use core::sync::atomic::{AtomicBool, Ordering};

use uacpi_wrapper::*;

static STARTED: AtomicBool = AtomicBool::new(false);
static READY: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct InitError {
    pub stage: &'static str,
    pub status: uacpi_status,
}

impl core::fmt::Display for InitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let status = unsafe { CStr::from_ptr(uacpi_status_to_string(self.status)) };
        write!(f, "{}: {}", self.stage, status.to_str().unwrap_or("unknown uACPI status"))
    }
}

fn check(stage: &'static str, status: uacpi_status) -> Result<(), InitError> {
    if status == UACPI_STATUS_OK {
        Ok(())
    } else {
        Err(InitError {
            stage,
            status,
        })
    }
}

/// Whether the namespace and runtime events are ready for device drivers.
pub fn is_initialized() -> bool {
    READY.load(Ordering::Acquire)
}

pub fn initialize_acpi() -> Result<(), InitError> {
    if STARTED.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
        return Err(InitError {
            stage: "duplicate initialization",
            status: UACPI_STATUS_ALREADY_EXISTS,
        });
    }
    super::uacpi_host::initialize();
    unsafe {
        check("subsystem initialization", uacpi_initialize(0))?;
        check("namespace load", uacpi_namespace_load())?;
        check("namespace initialization", uacpi_namespace_initialize())?;
        let model = cfg_select! {
            target_arch = "x86_64" => UACPI_INTERRUPT_MODEL_IOAPIC,
            target_arch = "aarch64" => UACPI_INTERRUPT_MODEL_GIC,
            target_arch = "riscv64" => UACPI_INTERRUPT_MODEL_RINTC,
        };
        check("interrupt model selection", uacpi_set_interrupt_model(model))?;
        check("GPE initialization", uacpi_finalize_gpe_initialization())?;
    }
    READY.store(true, Ordering::Release);
    #[cfg(feature = "acpi_self_test")]
    uacpi_host::self_test::run();
    Ok(())
}
