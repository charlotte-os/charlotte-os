use core::{
    iter::Step,
    ops::{
        Add,
        AddAssign,
        Sub,
    },
};

use crate::cpu::isa::{
    interface::memory::address::{
        Address,
        VirtualAddressIfce,
    },
    memory::address::VADDR_SIG_BITS,
};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress {
    raw: usize,
}

impl core::fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VAddr({:#x})", self.raw)
    }
}

/// VAddr component indexes and masks
const PAGE_TABLE_INDEX_MASK: usize = 0x1ff;
const PML4_INDEX_SHIFT: usize = 39;
const PML4_INDEX_MASK: usize = PAGE_TABLE_INDEX_MASK << PML4_INDEX_SHIFT;
const PDPT_INDEX_SHIFT: usize = 30;
const PDPT_INDEX_MASK: usize = PAGE_TABLE_INDEX_MASK << PDPT_INDEX_SHIFT;
const PD_INDEX_SHIFT: usize = 21;
const PD_INDEX_MASK: usize = PAGE_TABLE_INDEX_MASK << PD_INDEX_SHIFT;
const PT_INDEX_SHIFT: usize = 12;
const PT_INDEX_MASK: usize = PAGE_TABLE_INDEX_MASK << PT_INDEX_SHIFT;
const OFFSET_MASK: usize = 0xfff;

impl VirtualAddress {
    /// Convenience functions to get the index for each level of the page table hierarchy

    pub fn pml4_index(&self) -> usize {
        (self.raw & PML4_INDEX_MASK) >> PML4_INDEX_SHIFT
    }

    pub fn pdpt_index(&self) -> usize {
        (self.raw & PDPT_INDEX_MASK) >> PDPT_INDEX_SHIFT
    }

    pub fn pd_index(&self) -> usize {
        (self.raw & PD_INDEX_MASK) >> PD_INDEX_SHIFT
    }

    pub fn pt_index(&self) -> usize {
        (self.raw & PT_INDEX_MASK) >> PT_INDEX_SHIFT
    }

    pub fn page_offset(&self) -> usize {
        self.raw & OFFSET_MASK
    }

    /// Safety: The address must be valid and in canonical form
    pub const unsafe fn from_raw_unchecked(raw: usize) -> Self {
        VirtualAddress {
            raw,
        }
    }
}

impl Address for VirtualAddress {
    const MAX: Self = VirtualAddress {
        raw: usize::MAX,
    };
    const MIN: Self = VirtualAddress {
        raw: 0,
    };
    const NULL: Self = VirtualAddress {
        raw: 0,
    };

    fn is_aligned_to(&self, alignment: usize) -> bool {
        self.raw % alignment == 0
    }

    fn next_aligned_to(&self, alignment: usize) -> Self {
        let mask = alignment - 1;
        let aligned = (<VirtualAddress as Into<usize>>::into(*self) + mask) & !mask;
        VirtualAddress::from(aligned)
    }

    fn prev_aligned_to(&self, alignment: usize) -> Self {
        VirtualAddress {
            raw: if alignment % 2 == 0 {
                self.raw & !(alignment - 1)
            } else {
                self.raw - (self.raw % alignment)
            },
        }
    }

    fn is_valid(value: usize) -> bool {
        value != 0
    }

    fn is_null(&self) -> bool {
        self.raw == 0
    }

    unsafe fn from_unchecked(addr: usize) -> Self {
        VirtualAddress {
            raw: addr,
        }
    }
}

impl VirtualAddressIfce for VirtualAddress {
    fn from_ptr<T>(ptr: *const T) -> Self {
        VirtualAddress {
            raw: ptr as usize,
        }
    }

    fn from_mut<T>(ptr: *mut T) -> Self {
        VirtualAddress {
            raw: ptr as usize,
        }
    }

    fn into_ptr<T>(self) -> *const T {
        self.raw as *const T
    }

    fn into_mut<T>(self) -> *mut T {
        self.raw as *mut T
    }
}

impl From<usize> for VirtualAddress {
    fn from(value: usize) -> Self {
        let mask = (1 << *VADDR_SIG_BITS) - 1;
        let sign_extended = if value & (1 << (*VADDR_SIG_BITS - 1)) != 0 {
            value | (!mask)
        } else {
            value & mask
        };
        VirtualAddress {
            raw: sign_extended,
        }
    }
}

impl Into<usize> for VirtualAddress {
    fn into(self) -> usize {
        self.raw
    }
}

impl From<u64> for VirtualAddress {
    fn from(value: u64) -> Self {
        VirtualAddress::from(value as usize)
    }
}
impl Into<u64> for VirtualAddress {
    fn into(self) -> u64 {
        self.raw as u64
    }
}

impl Sub for VirtualAddress {
    type Output = isize;

    fn sub(self, other: Self) -> Self::Output {
        self.raw as isize - other.raw as isize
    }
}

impl Add<isize> for VirtualAddress {
    type Output = VirtualAddress;

    fn add(self, other: isize) -> Self::Output {
        VirtualAddress {
            raw: (self.raw as isize + other) as usize,
        }
    }
}

impl Sub<isize> for VirtualAddress {
    type Output = VirtualAddress;

    fn sub(self, other: isize) -> Self::Output {
        VirtualAddress::from(self.raw - other as usize)
    }
}

impl Add<usize> for VirtualAddress {
    type Output = VirtualAddress;

    fn add(self, other: usize) -> Self::Output {
        VirtualAddress::from(self.raw + other)
    }
}

impl Sub<usize> for VirtualAddress {
    type Output = VirtualAddress;

    fn sub(self, other: usize) -> Self::Output {
        VirtualAddress::from(self.raw - other)
    }
}

impl<T> AddAssign<T> for VirtualAddress
where
    VirtualAddress: Add<T, Output = VirtualAddress>,
{
    fn add_assign(&mut self, rhs: T) {
        *self = *self + rhs;
    }
}

impl Step for VirtualAddress {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        if start > end {
            (0, None)
        } else {
            (end.raw - start.raw, Some(end.raw - start.raw))
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        if start.raw.saturating_add(count) < usize::MAX {
            Some(VirtualAddress {
                raw: start.raw + count,
            })
        } else {
            None
        }
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        if start.raw.saturating_sub(count) > usize::MIN {
            Some(VirtualAddress {
                raw: start.raw - count,
            })
        } else {
            None
        }
    }

    fn forward_overflowing(start: Self, count: usize) -> (Self, bool) {
        let (new_raw, overflow) = start.raw.overflowing_add(count);
        (
            VirtualAddress {
                raw: new_raw,
            },
            overflow,
        )
    }

    fn backward_overflowing(start: Self, count: usize) -> (Self, bool) {
        let (new_raw, overflow) = start.raw.overflowing_sub(count);
        (
            VirtualAddress {
                raw: new_raw,
            },
            overflow,
        )
    }
}

impl Default for VirtualAddress {
    fn default() -> Self {
        VirtualAddress {
            raw: 0,
        }
    }
}
