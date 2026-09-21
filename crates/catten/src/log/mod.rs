//! # Kernel Logging Macros
//!
//! This module provides convenient macros for logging messages to the kernel
//! log. They will be updated as the kernel develops to provide more
//! functionality and use an actual kernel log that will reside in memory and be
//! stored in a file.
//!
//! There are two sinks behind the macros. The [early console](early) is a polled serial device at
//! a fixed, architecture-defined address; it needs neither the heap nor a framebuffer, so it is
//! available from the kernel's first statement and it is what [`early_log!`] and [`early_logln!`]
//! write to. The [framebuffer terminal](flanterm) is what a user of the machine actually reads,
//! but Flanterm allocates, so it cannot be touched until the heap is up.
//!
//! [`log!`] and [`logln!`] write to both. The duplication is deliberate: it makes the serial
//! transcript — which a host running the kernel under QEMU can capture to a file — the whole
//! kernel log rather than only the handful of lines that precede the heap.

mod chars;
pub mod early;
#[cfg(feature = "display")]
pub mod flanterm;

/// Logs to the early console only, without a trailing line break.
///
/// Valid at any point after control passes from the bootloader, including before the heap exists.
#[macro_export]
macro_rules! early_log {
    ($text:expr $(, $arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::log::early::EARLY_CONSOLE.lock(), $text $(, $arg)*);
    })
}

/// Logs a line to the early console only.
///
/// Valid at any point after control passes from the bootloader, including before the heap exists.
/// See [`logln!`] on why the line break is written by hand.
#[macro_export]
macro_rules! early_logln {
    ($text:expr $(, $arg:tt)*) => ({
        use core::fmt::Write;
        let mut console = $crate::log::early::EARLY_CONSOLE.lock();
        let _ = write!(console, $text $(, $arg)*);
        let _ = console.write_str("\r\n");
    })
}

#[macro_export]
macro_rules! log {
    ($text:expr $(, $arg:tt)*) => ({
        $crate::cpu::multiprocessor::interrupt_tracking::INT_STATE.save_int();
        #[cfg(feature = "display")]
        {
            use core::fmt::Write;
            let _ = write!($crate::log::flanterm::context().lock(), $text $(, $arg)*);
        }
        $crate::early_log!($text $(, $arg)*);
        $crate::cpu::multiprocessor::interrupt_tracking::INT_STATE.restore_int();
    })
}
/// Logs a line, terminated by a carriage return and a line feed.
///
/// `writeln!` is deliberately not used here: it can only append a bare `\n`, which the terminal
/// treats as a pure line feed and which would leave the next line starting in this line's last
/// column. The two characters are written under a single lock so that a line cannot be split by
/// another LP logging in between. A format string that spans several lines has to spell out
/// `\r\n` at each break itself.
#[macro_export]
macro_rules! logln {
    ($text:expr $(, $arg:tt)*) => ({
        $crate::cpu::multiprocessor::interrupt_tracking::INT_STATE.save_int();
        #[cfg(feature = "display")]
        {
            use core::fmt::Write;
            let mut ft_ctx = $crate::log::flanterm::context().lock();
            let _ = write!(ft_ctx, $text $(, $arg)*);
            let _ = ft_ctx.write_str("\r\n");
        }
        $crate::early_logln!($text $(, $arg)*);
        $crate::cpu::multiprocessor::interrupt_tracking::INT_STATE.restore_int();
    })
}
