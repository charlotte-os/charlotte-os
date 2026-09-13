//! # Early Log Console Backend for x86-64
//!
//! COM1 sits at I/O port `0x3f8` on every machine that traces its lineage to the PC/AT, which is
//! every x86-64 machine and every x86-64 emulator. No enumeration is needed to find it, so it is
//! reachable from the kernel's first instruction.

use core::arch::asm;
use core::sync::atomic::{AtomicBool, Ordering};

/// COM1's base I/O port.
const COM1: u16 = 0x3f8;

/* Register offsets from the base port. The first two change meaning when DLAB is set in the line
 * control register, at which point they address the two halves of the baud rate divisor. */
const DATA: u16 = 0;
const INT_ENABLE: u16 = 1;
const FIFO_CTRL: u16 = 2;
const LINE_CTRL: u16 = 3;
const MODEM_CTRL: u16 = 4;
const LINE_STATUS: u16 = 5;

/// Line control: the divisor latch access bit, which remaps the first two registers.
const LCR_DLAB: u8 = 1 << 7;
/// Line control: eight data bits, no parity, one stop bit.
const LCR_8N1: u8 = 0b11;
/// FIFO control: enable both FIFOs, clear them, and trigger the receiver at fourteen bytes.
const FCR_ENABLE_CLEAR: u8 = 0xc7;
/// Modem control: assert data terminal ready and request to send.
const MCR_DTR_RTS: u8 = 0b11;
/// Modem control: loop the transmitter back to the receiver, for the presence test.
const MCR_LOOPBACK: u8 = 0x1e;
/// Line status: the transmit holding register is empty and will accept another byte.
const LSR_THR_EMPTY: u8 = 1 << 5;
/// Line status: a received byte is waiting in the receive buffer.
const LSR_DATA_READY: u8 = 1 << 0;

/// Arbitrary byte sent through the loopback to see whether anything is listening.
const PRESENCE_TEST_BYTE: u8 = 0xae;

/// How many times to poll a status register before giving up on the port.
///
/// Nothing here can be allowed to spin forever on hardware that never answers, and the count is
/// large enough that a real UART at the slowest sane baud rate will always win the race.
const POLL_LIMIT: u32 = 100_000;

/// Whether COM1 answered the loopback test, and so whether writing to it is worth anything.
static PRESENT: AtomicBool = AtomicBool::new(false);

#[inline]
unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

pub(super) fn init() {
    unsafe {
        outb(COM1 + INT_ENABLE, 0x00);
        outb(COM1 + LINE_CTRL, LCR_DLAB);
        // Divisor one over the 115200 Hz base clock, so 115200 baud.
        outb(COM1 + DATA, 0x01);
        outb(COM1 + INT_ENABLE, 0x00);
        outb(COM1 + LINE_CTRL, LCR_8N1);
        outb(COM1 + FIFO_CTRL, FCR_ENABLE_CLEAR);
    }

    PRESENT.store(is_present(), Ordering::Relaxed);

    // SAFETY: plain port I/O. Leaving the port in its normal, non-looped state is worth doing even
    // when the test failed, since a port that is merely late is better left configured than looped.
    unsafe {
        outb(COM1 + MODEM_CTRL, MCR_DTR_RTS);
    }
}

/// Reports whether a UART answers at COM1, by looping the transmitter back to the receiver and
/// seeing whether a byte survives the round trip.
///
/// A board with no UART at `0x3f8` usually floats the bus and reads back `0xff`, but some chipsets
/// answer `0x00` instead, and that pattern would otherwise make every log byte burn the full poll
/// budget waiting on a transmitter that does not exist.
fn is_present() -> bool {
    unsafe {
        outb(COM1 + MODEM_CTRL, MCR_LOOPBACK);
        outb(COM1 + DATA, PRESENCE_TEST_BYTE);

        for _ in 0..POLL_LIMIT {
            if inb(COM1 + LINE_STATUS) & LSR_DATA_READY != 0 {
                return inb(COM1 + DATA) == PRESENCE_TEST_BYTE;
            }
            core::hint::spin_loop();
        }
    }

    false
}

pub(super) fn write_byte(byte: u8) {
    if !PRESENT.load(Ordering::Relaxed) {
        return;
    }

    unsafe {
        for _ in 0..POLL_LIMIT {
            if inb(COM1 + LINE_STATUS) & LSR_THR_EMPTY != 0 {
                outb(COM1 + DATA, byte);
                return;
            }
            core::hint::spin_loop();
        }
    }
}
