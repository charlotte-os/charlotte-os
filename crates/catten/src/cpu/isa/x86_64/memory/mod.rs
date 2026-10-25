pub mod address;
pub mod paging;
pub mod tlb;

pub use crate::cpu::isa::interface::memory::MemoryInterface;
use crate::cpu::isa::memory::address::paddr::PhysicalAddressError;
use crate::memory::{linear, physical};

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Unmapped,
    AlreadyMapped,
    NullVAddrNotAllowed,
    VAddrNotPageAligned,
    VAddrNotLargePageAligned,
    VAddrNotHugePageAligned,
    NoRequestedVAddrRegionAvailable,
    PhysicalMemoryError(physical::Error),
    VirtualMemoryError(linear::Error),
    SharedMemoryManagerError(physical::shared::Error),
}

impl From<physical::Error> for Error {
    fn from(err: physical::Error) -> Self {
        Error::PhysicalMemoryError(err)
    }
}

impl From<PhysicalAddressError> for Error {
    fn from(err: PhysicalAddressError) -> Self {
        Error::PhysicalMemoryError(physical::Error::PAddrError(err))
    }
}

impl From<linear::Error> for Error {
    fn from(err: linear::Error) -> Self {
        Error::VirtualMemoryError(err)
    }
}

impl From<physical::shared::Error> for Error {
    fn from(err: physical::shared::Error) -> Self {
        Error::SharedMemoryManagerError(err)
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
