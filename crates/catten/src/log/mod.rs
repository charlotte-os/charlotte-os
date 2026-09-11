//! # Kernel Logging Macros
//!
//! This module provides convenient macros for logging messages to the kernel
//! log. They will be updated as the kernel develops to provide more
//! functionality and use an actual kernel log that will reside in memory and be
//! stored in a file. For now they print to the framebuffer and can be observed directly on the
//! screen.

mod chars;
pub mod flanterm;

#[inline(always)]
pub fn early_save_interrupts() -> bool {
    #[cfg(target_arch = "x86_64")]
    let interrupts_were_enabled = crate::cpu::isa::lp::ops::get_int_state();

    #[cfg(not(target_arch = "x86_64"))]
    let interrupts_were_enabled = true;

    crate::cpu::isa::lp::ops::mask_interrupts!();
    interrupts_were_enabled
}

#[inline(always)]
pub fn early_restore_interrupts(interrupts_were_enabled: bool) {
    if interrupts_were_enabled {
        crate::cpu::isa::lp::ops::unmask_interrupts!();
    }
}

#[macro_export]
macro_rules! early_log {
    ($text:expr $(, $arg:tt)*) => {{
        // let interrupts_were_enabled = $crate::log::early_save_interrupts();
        // use core::fmt::Write;
        // let _ = write!($crate::log::flanterm::FT_CTX.lock(), $text $(, $arg)*);
        // $crate::log::early_restore_interrupts(interrupts_were_enabled);
    }};
}

#[macro_export]
macro_rules! early_logln {
    ($text:expr $(, $arg:tt)*) => {{
        // let interrupts_were_enabled = $crate::log::early_save_interrupts();
        // use core::fmt::Write;
        // let mut ft_ctx = $crate::log::flanterm::FT_CTX.lock();
        // let _ = write!(ft_ctx, $text $(, $arg)*);
        // let _ = ft_ctx.write_str("\r\n");
        // drop(ft_ctx);
        // $crate::log::early_restore_interrupts(interrupts_were_enabled);
    }};
}

#[macro_export]
macro_rules! log {
    ($text:expr $(, $arg:tt)*) => ({
        $crate::cpu::multiprocessor::interrupt_tracking::INT_STATE.save_int();
        use core::fmt::Write;
        let _ = write!($crate::log::flanterm::FT_CTX.lock(), $text $(, $arg)*);
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
        {
            use core::fmt::Write;
            let mut ft_ctx = $crate::log::flanterm::FT_CTX.lock();
            let _ = write!(ft_ctx, $text $(, $arg)*);
            let _ = ft_ctx.write_str("\r\n");
        }
        $crate::cpu::multiprocessor::interrupt_tracking::INT_STATE.restore_int();
    })
}
