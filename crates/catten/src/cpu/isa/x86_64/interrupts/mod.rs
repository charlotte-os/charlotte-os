//! # x86_64 Interrupt Management

pub mod dispatcher;
pub mod dynamic;
pub mod fixed;
pub mod idt;
pub mod ioapic;
pub mod x2apic;

use idt::*;
use spin::LazyLock;

pub type LocalIntCtlr = x2apic::X2Apic;

pub static GLOBAL_IDT: LazyLock<Idt> = LazyLock::new(|| Idt::new());

#[derive(Debug)]
pub enum Error {
    InvalidLpId,
    DynIhIdxInUse(usize),
    DynIhIdxInvalid(usize),
}
