use core::ops::Add;

pub use crate::cpu::isa::interface::io::{IReg64Ifce, OReg64Ifce};
use crate::memory::PhysicalAddress;
use crate::memory::physical::PhysicalAddressIfce;

#[derive(Copy, Clone, Debug)]
pub enum IoReg64 {
    Mmio(PhysicalAddress),
}

impl IReg64Ifce for IoReg64 {
    unsafe fn read(&self) -> u64 {
        match self {
            IoReg64::Mmio(address) => unsafe { core::ptr::read_volatile(address.into_hhdm_ptr()) },
        }
    }
}

impl OReg64Ifce for IoReg64 {
    unsafe fn write(&self, value: u64) {
        match self {
            IoReg64::Mmio(address) => unsafe {
                core::ptr::write_volatile(address.into_hhdm_mut(), value)
            },
        }
    }
}

impl Add<u16> for IoReg64 {
    type Output = IoReg64;

    fn add(self, rhs: u16) -> Self::Output {
        match self {
            IoReg64::Mmio(address) => IoReg64::Mmio(address + rhs as usize),
        }
    }
}
