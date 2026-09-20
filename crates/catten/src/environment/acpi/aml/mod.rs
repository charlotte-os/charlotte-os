//! # ACPI Machine Language (AML) Interpreter and Namespace Manager
mod uacpi_host;

use core::ffi::CStr;

use uacpi_wrapper::*;

fn uacpi_status_to_rs_string(status: uacpi_status) -> &'static CStr {
    unsafe { CStr::from_ptr(uacpi_status_to_string(status)) }
}

pub fn initialize_acpi() {
    let mut ret: uacpi_status;

    ret = unsafe { uacpi_initialize(0) };
    if core::hint::unlikely(ret != 0) {
        panic!("ACPI initialization failed: {}", uacpi_status_to_rs_string(ret).to_string_lossy());
    }

    ret = unsafe { uacpi_namespace_load() };
    if core::hint::unlikely(ret != 0) {
        panic!("ACPI namespace load failed: {}", uacpi_status_to_rs_string(ret).to_string_lossy());
    }

    ret = unsafe { uacpi_namespace_initialize() };
    if core::hint::unlikely(ret != 0) {
        panic!(
            "ACPI namespace initialization failed: {}",
            uacpi_status_to_rs_string(ret).to_string_lossy()
        );
    }

    unsafe {
        uacpi_set_interrupt_model(cfg_select! {
            target_arch = "x86_64" => UACPI_INTERRUPT_MODEL_IOAPIC,
            target_arch = "aarch64" => UACPI_INTERRUPT_MODEL_GIC,
            target_arch = "riscv64" => UACPI_INTERRUPT_MODEL_RINTC,
        });
    }

    ret = unsafe { uacpi_finalize_gpe_initialization() };
    if core::hint::unlikely(ret != 0) {
        panic!(
            "ACPI GPE initialization failed: {}",
            uacpi_status_to_rs_string(ret).to_string_lossy()
        );
    }
}
