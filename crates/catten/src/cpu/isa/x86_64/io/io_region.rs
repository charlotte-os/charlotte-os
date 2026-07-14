use crate::cpu::isa::io::IoReg8;
use crate::memory::{PhysicalAddress, VirtualAddress};

pub type IoPortAddr = u16;
pub type IoUSize = u16;

pub enum IoRegion {
    IoPort {
        start: IoPortAddr,
        len: IoUSize,
    },
    HhdmMmio {
        start: PhysicalAddress,
        len: usize,
    },
    MappedMmio {
        start: VirtualAddress,
        len: usize,
    },
}

impl IoRegion {
    pub fn len(&self) -> usize {
        match self {
            IoRegion::IoPort {
                len,
                ..
            } => *len as usize,
            IoRegion::HhdmMmio {
                len,
                ..
            } => *len,
            IoRegion::MappedMmio {
                len,
                ..
            } => *len,
        }
    }

    pub fn is_in_region(&self, reg: IoReg8) -> bool {
        match self {
            IoRegion::IoPort {
                start,
                len,
            } => {
                if let IoReg8::IoPort(addr) = reg {
                    addr >= *start && addr < (*start + *len)
                } else {
                    false
                }
            }
            IoRegion::HhdmMmio {
                start,
                len,
            } => {
                if let IoReg8::HhdmMmio(addr) = reg {
                    addr >= *start && addr < (*start + *len)
                } else {
                    false
                }
            }
            IoRegion::MappedMmio {
                start,
                len,
            } => {
                if let IoReg8::MappedMmio(addr) = reg {
                    addr >= *start && addr < (*start + *len)
                } else {
                    false
                }
            }
        }
    }
}
