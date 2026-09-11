//! # RISC-V Paging
//!
//! Catten targets Sv39 and wider translation modes, where the page size is 4 KiB and each further
//! level of the walk multiplies the mapping granularity by 512.

use crate::cpu::isa::interface::memory::{AddressSpaceInterface, MemoryInterface, MemoryMapping};
use crate::cpu::isa::memory::MemoryInterfaceImpl;
use crate::cpu::isa::memory::address::paddr::PhysicalAddress;
use crate::cpu::isa::memory::address::vaddr::VirtualAddress;
use crate::klib::size::{gibibytes, kibibytes, mebibytes};

/// The ASID field of `satp`, which is up to 16 bits wide depending on the implementation.
pub type HwAsid = u16;

pub const PAGE_SIZE: usize = 4096;
pub const N_PAGE_TABLE_ENTRIES: usize = 512;

/// A RISC-V address space, identified by the value loaded into `satp`.
pub struct AddressSpace {
    /// Supervisor Address Translation and Protection register: MODE, ASID and root PPN.
    satp: u64,
}

impl AddressSpace {
    pub fn get_satp(&self) -> u64 {
        self.satp
    }
}

impl AddressSpaceInterface for AddressSpace {
    const HUGE_PAGE_SIZE: usize = gibibytes(1);
    const LARGE_PAGE_SIZE: usize = mebibytes(2);
    const PAGE_SIZE: usize = kibibytes(4);

    fn get_current() -> Self {
        let satp: u64;
        unsafe {
            core::arch::asm!(
                "csrr {}, satp",
                out(reg) satp,
                options(nomem, nostack, preserves_flags)
            );
        }
        AddressSpace {
            satp,
        }
    }

    fn load(&self) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        unsafe {
            core::arch::asm!(
                "csrw satp, {}",
                "sfence.vma",
                in(reg) self.satp,
                options(nostack, preserves_flags)
            );
        }
        Ok(())
    }

    fn find_free_region(
        &mut self,
        _n_pages: usize,
        _range: (VirtualAddress, VirtualAddress),
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Walk the Sv39/Sv48 page tables looking for a free run of pages.")
    }

    fn find_free_region_large_aligned(
        &mut self,
        _n_large_pages: usize,
        _range: (VirtualAddress, VirtualAddress),
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Walk the page tables looking for a free run of megapages.")
    }

    fn find_free_region_huge_aligned(
        &mut self,
        _n_huge_pages: usize,
        _range: (VirtualAddress, VirtualAddress),
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Walk the page tables looking for a free run of gigapages.")
    }

    fn map_page(
        &mut self,
        _mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Install a leaf PTE for a 4 KiB page.")
    }

    fn unmap_page(
        &mut self,
        _vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Clear the leaf PTE for a 4 KiB page and return the frame it pointed at.")
    }

    fn map_large_page(
        &mut self,
        _mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Install a level 1 leaf PTE for a 2 MiB megapage.")
    }

    fn unmap_large_page(
        &mut self,
        _vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Clear the level 1 leaf PTE for a 2 MiB megapage.")
    }

    fn map_huge_page(
        &mut self,
        _mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Install a level 2 leaf PTE for a 1 GiB gigapage.")
    }

    fn unmap_huge_page(
        &mut self,
        _vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Clear the level 2 leaf PTE for a 1 GiB gigapage.")
    }

    fn is_mapped(
        &mut self,
        _vaddr: VirtualAddress,
    ) -> Result<bool, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Report whether a 4 KiB page is present.")
    }

    fn is_mapped_large_page(
        &mut self,
        _vaddr: VirtualAddress,
    ) -> Result<bool, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Report whether a 2 MiB megapage is present.")
    }

    fn is_mapped_huge_page(
        &mut self,
        _vaddr: VirtualAddress,
    ) -> Result<bool, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Report whether a 1 GiB gigapage is present.")
    }

    fn translate_address(
        &mut self,
        _vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Walk the page tables to resolve a virtual address to a physical one.")
    }
}
