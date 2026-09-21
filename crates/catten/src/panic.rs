//! # Rust Panic Handler

use core::fmt::Write;
use core::panic::PanicInfo;

use crate::cpu::isa::lp::ops::await_interrupt;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Panics may occur before the heap exists or while formatting under a console
    // lock. Report without allocating or recursively waiting on either sink.
    if let Some(mut serial) = crate::log::early::EARLY_CONSOLE.try_lock() {
        let _ = write!(serial, "***\r\nKernel panic: {}\r\n***\r\n", info);
    }
    #[cfg(feature = "display")]
    if let Some(display) = crate::log::flanterm::initialized_context() {
        if let Some(mut display) = display.try_lock() {
            let _ = write!(display, "***\r\nKernel panic: {}\r\n***\r\n", info);
        }
    }
    await_interrupt!();
}
