pub mod address;
pub mod paging;
pub mod tlb;

pub use crate::cpu::isa::interface::memory::MemoryInterface;
use crate::cpu::isa::memory::address::paddr::PhysicalAddressError;
use crate::memory::linear::Error as VMemError;
use crate::memory::physical::Error as PMemError;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Unmapped,
    AlreadyMapped,
    NullVAddrNotAllowed,
    VAddrNotPageAligned,
    VAddrNotLargePageAligned,
    VAddrNotHugePageAligned,
    NoRequestedVAddrRegionAvailable,
    PMemError(PMemError),
    VMemError(VMemError),
}

impl From<PMemError> for Error {
    fn from(err: PMemError) -> Self {
        Error::PMemError(err)
    }
}

impl From<PhysicalAddressError> for Error {
    fn from(err: PhysicalAddressError) -> Self {
        Error::PMemError(PMemError::PAddrError(err))
    }
}

impl From<VMemError> for Error {
    fn from(err: VMemError) -> Self {
        Error::VMemError(err)
    }
}
pub struct MemoryInterfaceImpl;

impl MemoryInterface for MemoryInterfaceImpl {
    type AddressSpace = paging::AddressSpace;
    type Error = Error;
    type PAddr = address::paddr::PhysicalAddress;
    type VAddr = address::vaddr::VirtualAddress;

    const PAGE_SIZE: usize = paging::PAGE_SIZE;
}
