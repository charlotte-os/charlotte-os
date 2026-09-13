//! # Early Log Console Backend for x86-64
//!
//! COM1 sits at I/O port `0x3f8` on every machine that traces its lineage to the PC/AT, which is
//! every x86-64 machine and every x86-64 emulator. No enumeration is needed to find it, so it is
//! reachable from the kernel's first instruction.

use core::arch::asm;

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
/// Line status: the transmit holding register is empty and will accept another byte.
const LSR_THR_EMPTY: u8 = 1 << 5;

/// How many times to poll the line status register before giving a byte up for lost.
///
/// A machine with no UART at `0x3f8` floats the bus high and reads back `0xff`, which has the
/// empty bit set and so never stalls, but a chipset that reads back zero instead would hang the
/// kernel here for want of hardware that was never there.
const TX_POLL_LIMIT: u32 = 100_000;

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
        outb(COM1 + MODEM_CTRL, MCR_DTR_RTS);
    }
}

pub(super) fn write_byte(byte: u8) {
    unsafe {
        for _ in 0..TX_POLL_LIMIT {
            if inb(COM1 + LINE_STATUS) & LSR_THR_EMPTY != 0 {
                outb(COM1 + DATA, byte);
                return;
            }
            core::hint::spin_loop();
        }
    }
}
