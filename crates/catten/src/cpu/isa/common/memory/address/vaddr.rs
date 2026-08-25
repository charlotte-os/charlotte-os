use core::iter::Step;
use core::ops::{Add, AddAssign, Sub};

use crate::cpu::isa::interface::memory::address::{Address, VirtualAddressIfce};
use crate::cpu::isa::memory::address::VADDR_SIG_BITS;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualAddress(usize);

impl core::fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VAddr({:#x})", self.0)
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
        (self.0 & PML4_INDEX_MASK) >> PML4_INDEX_SHIFT
    }

    pub fn pdpt_index(&self) -> usize {
        (self.0 & PDPT_INDEX_MASK) >> PDPT_INDEX_SHIFT
    }

    pub fn pd_index(&self) -> usize {
        (self.0 & PD_INDEX_MASK) >> PD_INDEX_SHIFT
    }

    pub fn pt_index(&self) -> usize {
        (self.0 & PT_INDEX_MASK) >> PT_INDEX_SHIFT
    }

    pub fn page_offset(&self) -> usize {
        self.0 & OFFSET_MASK
    }

    /// Safety: The address must be valid and in canonical form
    pub const unsafe fn from_raw_unchecked(raw: usize) -> Self {
        VirtualAddress(raw)
    }
}

impl Address for VirtualAddress {
    const MAX: Self = VirtualAddress(usize::MAX);
    const MIN: Self = VirtualAddress(0);
    const NULL: Self = VirtualAddress(0);

    fn is_aligned_to(&self, alignment: usize) -> bool {
        self.0 % alignment == 0
    }

    fn next_aligned_to(&self, alignment: usize) -> Self {
        let mask = alignment - 1;
        let aligned = (<VirtualAddress as Into<usize>>::into(*self) + mask) & !mask;
        VirtualAddress::from(aligned)
    }

    fn prev_aligned_to(&self, alignment: usize) -> Self {
        VirtualAddress(
            if alignment % 2 == 0 {
                self.0 & !(alignment - 1)
            } else {
                self.0 - (self.0 % alignment)
            },
        )
    }

    fn is_valid(value: usize) -> bool {
        value != 0
    }

    fn is_null(&self) -> bool {
        self.0 == 0
    }

    unsafe fn from_unchecked(addr: usize) -> Self {
        VirtualAddress(addr)
    }
}

impl VirtualAddressIfce for VirtualAddress {
    fn from_ptr<T>(ptr: *const T) -> Self {
        VirtualAddress(ptr as usize)
    }

    fn from_mut<T>(ptr: *mut T) -> Self {
        VirtualAddress(ptr as usize)
    }

    fn into_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    fn into_mut<T>(self) -> *mut T {
        self.0 as *mut T
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
        VirtualAddress(sign_extended)
    }
}

impl Into<usize> for VirtualAddress {
    fn into(self) -> usize {
        self.0
    }
}

impl From<u64> for VirtualAddress {
    fn from(value: u64) -> Self {
        VirtualAddress::from(value as usize)
    }
}
impl Into<u64> for VirtualAddress {
    fn into(self) -> u64 {
        self.0 as u64
    }
}

impl Sub for VirtualAddress {
    type Output = isize;

    fn sub(self, other: Self) -> Self::Output {
        self.0 as isize - other.0 as isize
    }
}

impl Add<isize> for VirtualAddress {
    type Output = VirtualAddress;

    fn add(self, other: isize) -> Self::Output {
        VirtualAddress((self.0 as isize + other) as usize)
    }
}

impl Sub<isize> for VirtualAddress {
    type Output = VirtualAddress;

    fn sub(self, other: isize) -> Self::Output {
        VirtualAddress::from(self.0 - other as usize)
    }
}

impl Add<usize> for VirtualAddress {
    type Output = VirtualAddress;

    fn add(self, other: usize) -> Self::Output {
        VirtualAddress::from(self.0 + other)
    }
}

impl Sub<usize> for VirtualAddress {
    type Output = VirtualAddress;

    fn sub(self, other: usize) -> Self::Output {
        VirtualAddress::from(self.0 - other)
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
            (end.0 - start.0, Some(end.0 - start.0))
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        if start.0.saturating_add(count) < usize::MAX {
            Some(VirtualAddress(start.0 + count))
        } else {
            None
        }
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        if start.0.saturating_sub(count) > usize::MIN {
            Some(VirtualAddress(start.0 - count))
        } else {
            None
        }
    }

    fn forward_overflowing(start: Self, count: usize) -> (Self, bool) {
        let (new_raw, overflow) = start.0.overflowing_add(count);
        (VirtualAddress(new_raw), overflow)
    }

    fn backward_overflowing(start: Self, count: usize) -> (Self, bool) {
        let (new_raw, overflow) = start.0.overflowing_sub(count);
        (VirtualAddress(new_raw), overflow)
    }
}

impl Default for VirtualAddress {
    fn default() -> Self {
        VirtualAddress(0)
    }
}
