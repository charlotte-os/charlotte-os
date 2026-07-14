pub mod address;

use crate::cpu::isa::memory::MemoryInterfaceImpl;
use crate::cpu::isa::memory::address::paddr::PhysicalAddress;
use crate::cpu::isa::memory::address::vaddr::VirtualAddress;
pub use crate::memory::linear::{MemoryMapping, PageType};

pub trait MemoryInterface {
    type VAddr: address::VirtualAddressIfce;
    type PAddr: address::PhysicalAddressIfce;
    type Error;
    type AddressSpace: AddressSpaceInterface;

    const PAGE_SIZE: usize;
}

pub trait AddressSpaceInterface {
    const PAGE_SIZE: usize;
    const LARGE_PAGE_SIZE: usize;
    const HUGE_PAGE_SIZE: usize;

    fn get_current() -> Self;
    fn load(&self) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn find_free_region(
        &mut self,
        n_pages: usize,
        range: (VirtualAddress, VirtualAddress),
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn find_free_region_large_aligned(
        &mut self,
        n_large_pages: usize,
        range: (VirtualAddress, VirtualAddress),
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn find_free_region_huge_aligned(
        &mut self,
        n_huge_pages: usize,
        range: (VirtualAddress, VirtualAddress),
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn map_page(
        &mut self,
        mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn unmap_page(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn map_large_page(
        &mut self,
        mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn unmap_large_page(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn map_huge_page(
        &mut self,
        mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn unmap_huge_page(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn is_mapped(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<bool, <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn is_mapped_large_page(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<bool, <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn is_mapped_huge_page(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<bool, <MemoryInterfaceImpl as MemoryInterface>::Error>;
    fn translate_address(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error>;
}
