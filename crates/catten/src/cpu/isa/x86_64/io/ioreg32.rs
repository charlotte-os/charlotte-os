use core::arch::asm;
use core::ops::Add;

pub use crate::cpu::isa::interface::io::{IReg32Ifce, OReg32Ifce};
use crate::memory::PhysicalAddress;
use crate::memory::physical::PhysicalAddressIfce;

#[derive(Copy, Clone, Debug)]
pub enum IoReg32 {
    IoPort(u16),
    Mmio(PhysicalAddress),
    PcieCfg {
        ecam_base: PhysicalAddress,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
    },
}

impl IReg32Ifce for IoReg32 {
    unsafe fn read(&self) -> u32 {
        match self {
            IoReg32::IoPort(port) => {
                let value: u32;
                unsafe {
                    asm!(
                        "in eax, dx",
                        in("dx") *port,
                        out("eax") value,
                    );
                }
                value
            }
            IoReg32::Mmio(address) => unsafe { core::ptr::read_volatile(address.into_hhdm_ptr()) },
            IoReg32::PcieCfg {
                ecam_base,
                bus,
                device,
                function,
                offset,
            } => {
                let phys_addr = *ecam_base
                    + ((*bus as usize) << 20)
                    + ((*device as usize) << 15)
                    + ((*function as usize) << 12)
                    + (*offset as usize);
                unsafe { core::ptr::read_volatile(phys_addr.into_hhdm_ptr()) }
            }
        }
    }
}

impl OReg32Ifce for IoReg32 {
    unsafe fn write(&self, value: u32) {
        match self {
            IoReg32::IoPort(port) => unsafe {
                asm!(
                    "out dx, eax",
                    in("dx") *port,
                    in("eax") value,
                );
            },
            IoReg32::Mmio(address) => unsafe {
                core::ptr::write_volatile(address.into_hhdm_mut(), value)
            },
            IoReg32::PcieCfg {
                ecam_base,
                bus,
                device,
                function,
                offset,
            } => {
                let phys_addr = *ecam_base
                    + ((*bus as usize) << 20)
                    + ((*device as usize) << 15)
                    + ((*function as usize) << 12)
                    + (*offset as usize);
                unsafe { core::ptr::write_volatile(phys_addr.into_hhdm_mut(), value) }
            }
        }
    }
}

impl Add<u16> for IoReg32 {
    type Output = IoReg32;

    fn add(self, rhs: u16) -> Self::Output {
        match self {
            IoReg32::IoPort(port) => IoReg32::IoPort(port.wrapping_add(rhs)),
            IoReg32::Mmio(address) => IoReg32::Mmio(address + rhs as usize),
            IoReg32::PcieCfg {
                ecam_base,
                bus,
                device,
                function,
                offset,
            } => IoReg32::PcieCfg {
                ecam_base,
                bus,
                device,
                function,
                offset: offset.wrapping_add(rhs),
            },
        }
    }
}
