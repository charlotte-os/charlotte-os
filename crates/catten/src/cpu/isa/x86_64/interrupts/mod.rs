//! # x86_64 Interrupt Management

pub mod dynamic;
pub mod fixed;
pub mod idt;
pub mod ioapic;
pub mod x2apic;

use idt::*;
use spin::{
    LazyLock,
    Mutex,
};

use crate::memory::IdTable;

pub type LocalIntCtlr = x2apic::X2Apic;

pub static BSP_IDT: Mutex<Idt> = Mutex::new(Idt::new());
pub static IDT_TABLE: LazyLock<IdTable<Mutex<Idt>>> = LazyLock::new(IdTable::new);

#[derive(Debug)]
pub enum Error {
    InvalidLpId,
    InvalidDynamicVectorNumber(u8),
}
