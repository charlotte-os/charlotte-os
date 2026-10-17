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
//!
//! # Deadlock Freedom
//!
//! Every macro formats its message into a buffer on the stack before taking any lock, so the
//! arguments are free to do anything a caller could do outside of a log statement: log
//! themselves, initialise a lazily initialised static that logs, allocate or take other locks.
//! Only the finished bytes are written to the sinks under a lock, and that lock records which LP
//! holds it so that an exception or NMI handler interrupting the holder can still log without
//! waiting on its own LP.

mod chars;
pub mod early;
#[cfg(feature = "display")]
pub mod flanterm;

use core::fmt::{self, Write};
use core::sync::atomic::{AtomicU32, Ordering};

use crate::cpu::isa::lp::LpId;
use crate::cpu::isa::lp::ops::{get_int_state, get_lp_id, mask_interrupts, unmask_interrupts};
use crate::cpu::multiprocessor::interrupt_tracking::INT_STATE;
use crate::cpu::multiprocessor::startup::ID_COUNTER;

/// Size of the on-stack buffer each log message is formatted into before it is written out.
///
/// Messages that fit, including the line break [`logln!`] appends, reach each sink in a single
/// write so that output from other LPs cannot land in the middle of them. Longer messages are
/// written in pieces that end at line breaks wherever the message has them.
const LOG_BUFFER_SIZE: usize = 512;

/// Which sinks a message is written to.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sinks {
    /// The early console only, which is usable before the heap and LP IDs exist.
    EarlyConsole,
    /// Every sink. Requires the heap and, on the first use, the LP count.
    All,
}

const NO_OWNER: LpId = LpId::MAX;
/// Stands in for the BSP's LP ID before any LP has been assigned one, at which point the BSP is
/// the only LP running.
const PRE_SMP_OWNER: LpId = LpId::MAX - 1;

/// The LP currently writing a message to the sinks, or [`NO_OWNER`].
///
/// This is the only lock a log message waits for, and it is only ever held while finished bytes
/// are copied to the sinks: no caller code, allocation or other lock acquisition happens under
/// it, so the LP holding it always makes progress. Recording the owner lets a log message from an
/// exception or NMI that interrupted the owner be recognised and written without waiting.
static EMIT_OWNER: AtomicU32 = AtomicU32::new(NO_OWNER);

fn current_owner_token() -> LpId {
    // LP IDs are assigned before any LP other than the BSP is started, so until the first one is
    // the BSP can be identified without asking the ISA, which may not be able to answer yet.
    if ID_COUNTER.load(Ordering::Acquire) == 0 {
        PRE_SMP_OWNER
    } else {
        get_lp_id()
    }
}

/// Writes finished text to the requested sinks.
fn emit(text: &str, sinks: Sinks) {
    if text.is_empty() {
        return;
    }
    let me = current_owner_token();
    // The owner masks interrupts, so seeing itself as the owner means this LP is running an
    // exception or NMI handler that interrupted the owner part way through writing.
    if EMIT_OWNER.load(Ordering::Acquire) == me {
        emit_reentrant(text, sinks);
        return;
    }
    // Anything lazily initialised is touched before the lock is taken so that initialising it,
    // which may allocate or log, never happens while other LPs wait on this one.
    #[cfg(feature = "display")]
    let display = (sinks == Sinks::All).then(flanterm::context);
    // `log!` and `logln!` require the heap, so they nest with other interrupt masking through
    // `INT_STATE`. The early macros must work before `INT_STATE` can be allocated, so they save
    // and restore the interrupt flag themselves.
    let saved_int_state = match sinks {
        Sinks::All => {
            INT_STATE.save_int();
            None
        }
        Sinks::EarlyConsole => {
            let int_state = get_int_state();
            mask_interrupts!();
            Some(int_state)
        }
    };
    while EMIT_OWNER
        .compare_exchange_weak(NO_OWNER, me, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
    #[cfg(feature = "display")]
    if let Some(display) = display {
        let _ = display.lock().write_str(text);
    }
    let _ = early::EARLY_CONSOLE.lock().write_str(text);
    EMIT_OWNER.store(NO_OWNER, Ordering::Release);
    match saved_int_state {
        None => INT_STATE.restore_int(),
        Some(true) => unmask_interrupts!(),
        Some(false) => {}
    }
}

/// Writes text from a handler that interrupted this LP while it was emitting, without waiting
/// for anything the interrupted code holds.
fn emit_reentrant(text: &str, sinks: Sinks) {
    #[cfg(feature = "display")]
    if sinks == Sinks::All
        && let Some(display) = flanterm::initialized_context()
        && let Some(mut display) = display.try_lock()
    {
        let _ = display.write_str(text);
    }
    #[cfg(not(feature = "display"))]
    let _ = sinks;
    match early::EARLY_CONSOLE.try_lock() {
        Some(mut console) => {
            let _ = console.write_str(text);
        }
        None => early::write_str_unlocked(text),
    }
}

/// Formats a message on the stack so that the caller's formatting code, which may itself log,
/// allocate or take locks, runs before anything is locked for writing.
struct LogBuffer {
    bytes: [u8; LOG_BUFFER_SIZE],
    len: usize,
    sinks: Sinks,
}

impl LogBuffer {
    const fn new(sinks: Sinks) -> Self {
        Self {
            bytes: [0; LOG_BUFFER_SIZE],
            len: 0,
            sinks,
        }
    }

    fn emit_prefix(&mut self, end: usize) {
        // Only whole `str`s split at character boundaries and line feeds are ever stored, so the
        // prefix is always valid UTF-8.
        if let Ok(text) = core::str::from_utf8(&self.bytes[..end]) {
            emit(text, self.sinks);
        }
        self.bytes.copy_within(end..self.len, 0);
        self.len -= end;
    }

    /// Makes room by writing out everything up to the last line break, or everything if there is
    /// none, so that long multi-line messages only interleave with other output between lines.
    fn flush(&mut self) {
        let end =
            self.bytes[..self.len].iter().rposition(|&b| b == b'\n').map_or(self.len, |i| i + 1);
        self.emit_prefix(end);
    }

    fn finish(mut self) {
        let len = self.len;
        self.emit_prefix(len);
    }
}

impl Write for LogBuffer {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while !s.is_empty() {
            let mut n = s.len().min(LOG_BUFFER_SIZE - self.len);
            while !s.is_char_boundary(n) {
                n -= 1;
            }
            if n == 0 {
                self.flush();
                continue;
            }
            self.bytes[self.len..self.len + n].copy_from_slice(&s.as_bytes()[..n]);
            self.len += n;
            s = &s[n..];
        }
        Ok(())
    }
}

/// Backs the logging macros; use those instead.
#[doc(hidden)]
pub fn write_log(args: fmt::Arguments, sinks: Sinks, newline: bool) {
    let mut buffer = LogBuffer::new(sinks);
    let _ = buffer.write_fmt(args);
    if newline {
        let _ = buffer.write_str("\r\n");
    }
    buffer.finish();
}

/// Logs to the early console only, without a trailing line break.
///
/// Valid at any point after control passes from the bootloader, including before the heap exists.
#[macro_export]
macro_rules! early_log {
    ($($arg:tt)*) => {
        $crate::log::write_log(format_args!($($arg)*), $crate::log::Sinks::EarlyConsole, false)
    };
}

/// Logs a line to the early console only.
///
/// Valid at any point after control passes from the bootloader, including before the heap exists.
/// See [`logln!`] on why the line break is written by hand.
#[macro_export]
macro_rules! early_logln {
    ($($arg:tt)*) => {
        $crate::log::write_log(format_args!($($arg)*), $crate::log::Sinks::EarlyConsole, true)
    };
}

/// Logs to every sink, without a trailing line break.
///
/// Requires the heap.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::log::write_log(format_args!($($arg)*), $crate::log::Sinks::All, false)
    };
}

/// Logs a line to every sink, terminated by a carriage return and a line feed.
///
/// `writeln!` is deliberately not used here: it can only append a bare `\n`, which the terminal
/// treats as a pure line feed and which would leave the next line starting in this line's last
/// column. The line break is formatted into the same buffer as the line itself so that another LP
/// cannot log in between them. A format string that spans several lines has to spell out `\r\n`
/// at each break itself.
///
/// Requires the heap.
#[macro_export]
macro_rules! logln {
    ($($arg:tt)*) => {
        $crate::log::write_log(format_args!($($arg)*), $crate::log::Sinks::All, true)
    };
}
