pub mod address_map;

pub use crate::cpu::isa::memory::address::paddr::PhysicalAddress;
pub use crate::cpu::isa::memory::address::vaddr::VirtualAddress;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidPageAttributes,
}

/// Represents the usage type of a memory page independent of the underlying hardware or
/// architecture.
/// This abstraction encodes the access permissions and cache attributes of a virtual memory page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    NotPresent,
    KernelCode,         //read, execute; writeback
    KernelData,         //read, write; writeback
    KernelRoData,       //read only; write-protected
    UserCode,           //user, read, execute; writeback
    UserData,           //user, read, write; writeback
    UserRoData,         //user, read only; write-protected
    Mmio,               //read, write, no caching; strong uncacheable
    DirectMemoryAccess, //read, write, no caching; writeback
    Framebuffer,        //read, write; write combining
}

impl PageType {
    pub fn is_global(&self) -> bool {
        match *self {
            PageType::KernelCode
            | PageType::KernelData
            | PageType::KernelRoData
            | PageType::Mmio
            | PageType::DirectMemoryAccess
            | PageType::Framebuffer => true,
            _ => false,
        }
    }

    pub fn is_user_accessible(&self) -> bool {
        match *self {
            PageType::UserCode | PageType::UserData | PageType::UserRoData => true,
            _ => false,
        }
    }

    pub fn is_writable(&self) -> bool {
        match *self {
            PageType::KernelData
            | PageType::UserData
            | PageType::Mmio
            | PageType::DirectMemoryAccess
            | PageType::Framebuffer => true,
            _ => false,
        }
    }

    pub fn is_no_execute(&self) -> bool {
        match *self {
            PageType::KernelCode | PageType::UserCode => false,
            _ => true,
        }
    }

    pub fn is_uncacheable(&self) -> bool {
        match *self {
            PageType::Mmio | PageType::DirectMemoryAccess | PageType::Framebuffer => true,
            _ => false,
        }
    }

    pub fn should_combine_writes(&self) -> bool {
        if *self == PageType::Framebuffer {
            true
        } else {
            false
        }
    }
}
#[derive(Debug, Clone)]
pub struct MemoryMapping {
    pub vaddr: VirtualAddress,
    pub paddr: PhysicalAddress,
    pub page_type: PageType,
}
