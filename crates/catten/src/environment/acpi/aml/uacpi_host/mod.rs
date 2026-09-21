//! Kernel services for the vendored uACPI 6.1 host ABI.
//!
//! The wrapper uses the full interpreter with preformatted logging, unsized frees,
//! and uACPI's built-in MMIO/zeroed-allocation helpers. Keep these exports in sync
//! with `vendor/uacpi/include/uacpi/kernel_api.h` when updating the library.
mod c_alloc;
mod interrupts;
mod io;
mod memory;
mod pci;
#[cfg(feature = "acpi_self_test")]
pub(super) mod self_test;
mod sync;
mod time;
mod work;

use core::ffi::{CStr, c_char};

use uacpi_wrapper::*;

use crate::environment::acpi::table_map::is_acpi_available;
use crate::logln;
use crate::memory::PhysicalAddress;

/// Prepare the deferred executor before uACPI installs any event handlers.
pub(super) fn initialize() {
    work::initialize();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_get_rsdp(out: *mut uacpi_phys_addr) -> uacpi_status {
    if out.is_null() {
        return UACPI_STATUS_INVALID_ARGUMENT;
    }
    if !is_acpi_available() {
        return UACPI_STATUS_NOT_FOUND;
    }
    unsafe {
        out.write(<PhysicalAddress as Into<u64>>::into(*crate::environment::acpi::RSDP_ADDR))
    };
    UACPI_STATUS_OK
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_log(level: uacpi_log_level, message: *const c_char) {
    if message.is_null() {
        return;
    }
    let level = match level {
        UACPI_LOG_DEBUG => "DEBUG",
        UACPI_LOG_INFO => "INFO",
        UACPI_LOG_WARN => "WARN",
        UACPI_LOG_ERROR => "ERROR",
        _ => "UNKNOWN",
    };
    let message = unsafe { CStr::from_ptr(message) }.to_str().unwrap_or("<invalid UTF-8>");
    // Both serial and framebuffer consoles require CRLF. Avoid duplicating uACPI's
    // trailing newline, and normalize embedded newlines without heap allocation.
    for line in message.trim_end_matches(['\r', '\n']).split('\n') {
        logln!("[ACPI][uACPI] {}: {}", level, (line.trim_end_matches('\r')));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_handle_firmware_request(
    request: *mut uacpi_firmware_request,
) -> uacpi_status {
    let Some(request) = (unsafe { request.as_ref() }) else {
        return UACPI_STATUS_INVALID_ARGUMENT;
    };
    match request.type_ as u32 {
        UACPI_FIRMWARE_REQUEST_TYPE_BREAKPOINT => {
            logln!("[ACPI][uACPI] AML breakpoint (no firmware debugger attached)");
            UACPI_STATUS_OK
        }
        UACPI_FIRMWARE_REQUEST_TYPE_FATAL => {
            let fatal = unsafe { request.__bindgen_anon_1.fatal };
            logln!(
                "[ACPI][uACPI] AML Fatal: type {}, code {:#x}, argument {:#x}",
                (fatal.type_),
                (fatal.code),
                (fatal.arg)
            );
            UACPI_STATUS_INTERNAL_ERROR
        }
        _ => UACPI_STATUS_INVALID_ARGUMENT,
    }
}
