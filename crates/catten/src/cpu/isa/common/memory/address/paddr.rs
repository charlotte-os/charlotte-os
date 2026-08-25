use core::ops::{Add, AddAssign, Sub};

use crate::cpu::isa::interface::memory::address::{
    Address,
    PhysicalAddressIfce,
    VirtualAddressIfce,
};
use crate::cpu::isa::memory::address::PADDR_MASK;
use crate::memory::HHDM_BASE;

#[derive(Debug, Clone, Copy)]
pub enum PhysicalAddressError {
    OutOfCpuSupportedRange(usize),
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress {
    raw: usize,
}

impl Address for PhysicalAddress {
    const MAX: Self = PhysicalAddress {
        raw: usize::MAX,
    };
    const MIN: Self = PhysicalAddress {
        raw: 0,
    };
    const NULL: Self = PhysicalAddress {
        raw: 0,
    };

    fn is_aligned_to(&self, alignment: usize) -> bool {
        self.raw % alignment == 0
    }

    fn is_valid(value: usize) -> bool {
        value & *PADDR_MASK == value
    }

    fn is_null(&self) -> bool {
        self.raw == 0
    }

    fn next_aligned_to(&self, alignment: usize) -> Self {
        unsafe { PhysicalAddress::from_unchecked(self.raw + (alignment - (self.raw % alignment))) }
    }

    fn prev_aligned_to(&self, alignment: usize) -> Self {
        PhysicalAddress {
            raw: if alignment % 2 == 0 {
                self.raw & !(alignment - 1)
            } else {
                self.raw - (self.raw % alignment)
            },
        }
    }

    unsafe fn from_unchecked(raw: usize) -> Self {
        PhysicalAddress {
            raw,
        }
    }
}

impl PhysicalAddressIfce for PhysicalAddress {
    unsafe fn into_hhdm_ptr<T>(self) -> *const T {
        (*HHDM_BASE).into_ptr::<T>().wrapping_byte_add(self.raw)
    }

    unsafe fn into_hhdm_mut<T>(self) -> *mut T {
        (*HHDM_BASE).into_mut::<T>().wrapping_byte_add(self.raw)
    }
}

impl<T> Into<*const T> for PhysicalAddress {
    fn into(self) -> *const T {
        (*HHDM_BASE).into_ptr::<T>().wrapping_byte_add(self.raw)
    }
}

impl<T> Into<*mut T> for PhysicalAddress {
    fn into(self) -> *mut T {
        (*HHDM_BASE).into_mut::<T>().wrapping_byte_add(self.raw)
    }
}

impl TryFrom<usize> for PhysicalAddress {
    type Error = PhysicalAddressError;

    fn try_from(value: usize) -> Result<Self, PhysicalAddressError> {
        if value & !*PADDR_MASK != 0 {
            Err(PhysicalAddressError::OutOfCpuSupportedRange(value))
        } else {
            Ok(PhysicalAddress {
                raw: value,
            })
        }
    }
}

impl Into<usize> for PhysicalAddress {
    fn into(self) -> usize {
        self.raw
    }
}

impl From<u64> for PhysicalAddress {
    fn from(value: u64) -> Self {
        PhysicalAddress {
            raw: value as usize & *PADDR_MASK,
        }
    }
}

impl Into<u64> for PhysicalAddress {
    fn into(self) -> u64 {
        self.raw as u64
    }
}

impl Add<isize> for PhysicalAddress {
    type Output = PhysicalAddress;

    fn add(self, rhs: isize) -> Self::Output {
        PhysicalAddress::try_from(self.raw.wrapping_add(rhs as usize)).unwrap()
    }
}

impl<T> AddAssign<T> for PhysicalAddress
where
    PhysicalAddress: Add<T, Output = PhysicalAddress>,
{
    fn add_assign(&mut self, rhs: T) {
        *self = *self + rhs;
    }
}

impl Sub<isize> for PhysicalAddress {
    type Output = PhysicalAddress;

    fn sub(self, rhs: isize) -> Self::Output {
        PhysicalAddress::try_from(self.raw.wrapping_sub(rhs as usize)).unwrap()
    }
}

impl Add<usize> for PhysicalAddress {
    type Output = PhysicalAddress;

    fn add(self, rhs: usize) -> Self::Output {
        PhysicalAddress::try_from(self.raw.wrapping_add(rhs)).unwrap()
    }
}

impl Sub<usize> for PhysicalAddress {
    type Output = PhysicalAddress;

    fn sub(self, rhs: usize) -> Self::Output {
        PhysicalAddress::try_from(self.raw.wrapping_sub(rhs)).unwrap()
    }
}

impl Default for PhysicalAddress {
    fn default() -> Self {
        PhysicalAddress {
            raw: 0,
        }
    }
}
