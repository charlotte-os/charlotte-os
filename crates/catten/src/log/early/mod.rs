//! # Early Kernel Log Console
//!
//! The framebuffer terminal that carries the kernel log for most of the system's life cannot be
//! reached until the heap exists: Flanterm allocates its canvas and its glyph cache up front. That
//! leaves everything before [`init_primary_allocator`] — the ISA bring-up, the physical frame
//! allocator, the heap's own construction — with nowhere to say anything, which is precisely the
//! window in which a boot most often goes wrong.
//!
//! This module fills that window with a character device that needs no memory beyond the few
//! bytes of state below and no configuration beyond a fixed address the platform has guaranteed
//! since before the kernel was loaded:
//!
//! - **x86-64** writes to COM1 at I/O port `0x3f8`, the location the PC/AT fixed in 1984.
//! - **AArch64** writes to the PL011 the `virt` machine and the SBSA platforms place at physical
//!   `0x0900_0000`, reached through the bootloader's higher half direct map.
//! - **RISC-V** hands each byte to the SEE through an `ecall`, so the console follows whatever the
//!   firmware already decided it was.
//!
//! All three are the port QEMU exposes as its first serial device, so pointing `-serial` at a
//! chardev with a `logfile=` is enough to land the whole kernel log in a file on the host. See the
//! `qemu-run-*` recipes in the `Justfile`.
//!
//! The [`log!`](crate::log) and [`logln!`](crate::logln) macros keep writing here after the
//! framebuffer terminal comes up, so the transcript in that file is the complete log rather than
//! just its first few lines.
//!
//! [`init_primary_allocator`]: crate::memory::allocators::global_allocator::init_primary_allocator

use core::fmt::Write;

use crate::cpu::multiprocessor::spin::mutex::Mutex;

cfg_select! {
    target_arch = "aarch64" => {
        mod aarch64;
        use aarch64 as backend;
    }
    target_arch = "riscv64" => {
        mod riscv64;
        use riscv64 as backend;
    }
    target_arch = "x86_64" => {
        mod x86_64;
        use x86_64 as backend;
    }
}

/// A polled, allocation-free character device that the kernel log can reach at any point after
/// control passes from the bootloader.
///
/// # Line endings
///
/// Bytes go out exactly as they arrive. A terminal on the far end of a serial line treats a line
/// feed as a line feed and nothing else, so every line break written here is `\r\n`, matching the
/// convention the framebuffer terminal already imposes.
pub struct EarlyConsole {
    initialised: bool,
}

impl EarlyConsole {
    const fn new() -> Self {
        EarlyConsole {
            initialised: false,
        }
    }
}

impl Write for EarlyConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Initialising on the first write rather than from an explicit call keeps the console
        // usable from the kernel's very first statement, before anything has had a chance to run.
        if !self.initialised {
            backend::init();
            self.initialised = true;
        }

        for byte in s.as_bytes() {
            backend::write_byte(*byte);
        }

        Ok(())
    }
}

/// The kernel's early log console.
///
/// The mutex masks interrupts on the locking LP for as long as it is held, so a line cannot be
/// interleaved with one from an interrupt handler on the same processor or from another LP.
pub static EARLY_CONSOLE: Mutex<EarlyConsole> = Mutex::new(EarlyConsole::new());
