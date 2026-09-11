//! # Rust Panic Handler

use core::panic::PanicInfo;

use crate::cpu::isa::lp::ops::await_interrupt;
use crate::logln;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    logln!("***\r\nA kernel panic has occurred with the following cause:\r\n{}\r\n***", _info);
    await_interrupt!();
}
