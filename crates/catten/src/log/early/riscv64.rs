//! # Early Log Console Backend for RISC-V
//!
//! A RISC-V kernel runs in supervisor mode underneath a supervisor execution environment, and that
//! environment already owns a console: it is where the firmware printed its own boot messages.
//! Asking the SEE to put a byte there through an `ecall` is both simpler and more portable than
//! locating a UART, because it works on any platform with a conforming SBI implementation rather
//! than only on the ones that happen to put an NS16550 at a known address.
//!
//! Two extensions can do it. The debug console extension (`DBCN`) is the one SBI 2.0 defines for
//! the purpose; the legacy `console_putchar` call it replaced is deprecated but is all that older
//! implementations offer. [`init`] probes for the former and falls back to the latter.

use core::arch::asm;
use core::sync::atomic::{AtomicBool, Ordering};

/// Base extension, which every SBI implementation provides.
const EID_BASE: usize = 0x10;
/// Base extension: report whether the given extension ID is implemented.
const FID_PROBE_EXTENSION: usize = 3;

/// Debug console extension, added in SBI 2.0.
const EID_DBCN: usize = 0x4442434e;
/// Debug console extension: write a single byte, passed by value.
const FID_DBCN_WRITE_BYTE: usize = 2;

/// Legacy console extension, deprecated but still widely implemented.
const EID_LEGACY_CONSOLE_PUTCHAR: usize = 0x01;

static USE_DBCN: AtomicBool = AtomicBool::new(false);

/// Issues an SBI call, returning the error code and the return value the SEE leaves in `a0` and
/// `a1`.
///
/// The legacy extensions predate that convention and return a single value in `a0`, which this
/// reports as the error; none of the calls made here inspect it.
#[inline]
fn sbi_call(eid: usize, fid: usize, arg0: usize) -> (isize, usize) {
    let error: isize;
    let value: usize;

    // SAFETY: an `ecall` traps to the execution environment, which returns to the following
    // instruction with only the caller-saved registers named here disturbed.
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") arg0 => error,
            lateout("a1") value,
            in("a6") fid,
            in("a7") eid,
            options(nostack)
        );
    }

    (error, value)
}

pub(super) fn init() {
    let (error, value) = sbi_call(EID_BASE, FID_PROBE_EXTENSION, EID_DBCN);
    USE_DBCN.store(error == 0 && value != 0, Ordering::Relaxed);
}

pub(super) fn write_byte(byte: u8) {
    if USE_DBCN.load(Ordering::Relaxed) {
        sbi_call(EID_DBCN, FID_DBCN_WRITE_BYTE, byte as usize);
    } else {
        sbi_call(EID_LEGACY_CONSOLE_PUTCHAR, 0, byte as usize);
    }
}
