use core::arch::asm;
use core::ops::Add;

pub use crate::cpu::isa::interface::io::{IReg16Ifce, OReg16Ifce};
use crate::memory::PhysicalAddress;
use crate::memory::physical::PhysicalAddressIfce;

#[derive(Copy, Clone, Debug)]
pub enum IoReg16 {
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

impl IReg16Ifce for IoReg16 {
    unsafe fn read(&self) -> u16 {
        match self {
            IoReg16::IoPort(port) => {
                let value: u16;
                unsafe {
                    asm!(
                        "in ax, dx",
                        in("dx") *port,
                        out("ax") value,
                    );
                }
                value
            }
            IoReg16::Mmio(address) => unsafe { core::ptr::read_volatile(address.into_hhdm_ptr()) },
            IoReg16::PcieCfg {
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

impl OReg16Ifce for IoReg16 {
    unsafe fn write(&self, value: u16) {
        match self {
            IoReg16::IoPort(port) => unsafe {
                asm!(
                    "out dx, ax",
                    in("dx") *port,
                    in("ax") value,
                );
            },
            IoReg16::Mmio(address) => unsafe {
                core::ptr::write_volatile(address.into_hhdm_mut(), value)
            },
            IoReg16::PcieCfg {
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

impl Add<u16> for IoReg16 {
    type Output = IoReg16;

    fn add(self, rhs: u16) -> Self::Output {
        match self {
            IoReg16::IoPort(port) => IoReg16::IoPort(port.wrapping_add(rhs)),
            IoReg16::Mmio(address) => IoReg16::Mmio(address + rhs as usize),
            IoReg16::PcieCfg {
                ecam_base,
                bus,
                device,
                function,
                offset,
            } => IoReg16::PcieCfg {
                ecam_base,
                bus,
                device,
                function,
                offset: offset.wrapping_add(rhs),
            },
        }
    }
}
