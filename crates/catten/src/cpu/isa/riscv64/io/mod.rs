//! # RISC-V Input/Output
//!
//! RISC-V has no separate I/O address space, so every device register is reached through MMIO.

use core::ops::Add;

use crate::cpu::isa::interface::io::{IReg8Ifce, OReg8Ifce};

#[derive(Copy, Clone, Debug)]
pub struct IoReg8 {
    address: *mut u8,
}

impl IoReg8 {
    /// # Safety
    ///
    /// `address` must be a mapped MMIO device register that stays valid for as long as this
    /// register handle is used.
    pub const unsafe fn new(address: *mut u8) -> Self {
        IoReg8 {
            address,
        }
    }
}

impl IReg8Ifce for IoReg8 {
    unsafe fn read(&self) -> u8 {
        unsafe { core::ptr::read_volatile(self.address) }
    }
}

impl OReg8Ifce for IoReg8 {
    unsafe fn write(&self, value: u8) {
        unsafe { core::ptr::write_volatile(self.address, value) }
    }
}

impl Add<u16> for IoReg8 {
    type Output = IoReg8;

    fn add(self, rhs: u16) -> Self::Output {
        IoReg8 {
            address: unsafe { self.address.add(rhs as usize) },
        }
    }
}
