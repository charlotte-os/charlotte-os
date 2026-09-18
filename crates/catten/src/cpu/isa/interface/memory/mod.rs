pub mod address;

use crate::cpu::isa::memory::MemoryInterfaceImpl;
use crate::cpu::isa::memory::address::paddr::PhysicalAddress;
use crate::cpu::isa::memory::address::vaddr::VirtualAddress;
use crate::memory::allocators::memory::PageSize;
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

    fn map_range(
        &mut self,
        phys_base: PhysicalAddress,
        virt_base: VirtualAddress,
        page_type: PageType,
        page_size: PageSize,
        num_pages: usize,
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        let mapping_function = match page_size {
            PageSize::Standard => Self::map_page,
            PageSize::Large => Self::map_large_page,
            PageSize::Huge => Self::map_huge_page,
        };
        for n in 0..num_pages {
            let mapping = MemoryMapping {
                vaddr: virt_base + n * page_size.num_bytes(),
                paddr: phys_base + n * page_size.num_bytes(),
                page_type,
            };
            mapping_function(self, mapping)?;
        }
        Ok(virt_base)
    }

    fn find_and_map_range(
        &mut self,
        phys_base: PhysicalAddress,
        page_type: PageType,
        page_size: PageSize,
        num_pages: usize,
        range: (VirtualAddress, VirtualAddress),
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        let find_function = match page_size {
            PageSize::Standard => Self::find_free_region,
            PageSize::Large => Self::find_free_region_large_aligned,
            PageSize::Huge => Self::find_free_region_huge_aligned,
        };
        let virt_base = find_function(self, num_pages, range)?;
        self.map_range(phys_base, virt_base, page_type, page_size, num_pages)
    }
}
