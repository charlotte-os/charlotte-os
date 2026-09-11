//! # AArch64 Paging
//!
//! Catten uses the 4 KiB translation granule, where a level 2 block maps 2 MiB and a level 1 block
//! maps 1 GiB. `TTBR0_EL1` holds the user half of the address space and `TTBR1_EL1` the kernel
//! half.

use core::arch::asm;

use crate::cpu::isa::interface::memory::{AddressSpaceInterface, MemoryInterface, MemoryMapping};
use crate::cpu::isa::memory::MemoryInterfaceImpl;
use crate::cpu::isa::memory::address::paddr::PhysicalAddress;
use crate::cpu::isa::memory::address::vaddr::VirtualAddress;
use crate::klib::size::{gibibytes, kibibytes, mebibytes};

/// The ASID field of `TTBR0_EL1`, which is 8 or 16 bits wide depending on `TCR_EL1.AS`.
pub type HwAsid = u16;

pub const PAGE_SIZE: usize = kibibytes(4);
pub const N_PAGE_TABLE_ENTRIES: usize = 512;

pub struct AddressSpace {
    /// User space translation table base register.
    ttbr0_el1: u64,
    /// Kernel space translation table base register.
    ttbr1_el1: u64,
}

impl AddressSpace {
    pub fn get_ttbr0(&self) -> u64 {
        self.ttbr0_el1
    }

    pub fn get_ttbr1(&self) -> u64 {
        self.ttbr1_el1
    }
}

impl AddressSpaceInterface for AddressSpace {
    const HUGE_PAGE_SIZE: usize = gibibytes(1);
    const LARGE_PAGE_SIZE: usize = mebibytes(2);
    const PAGE_SIZE: usize = kibibytes(4);

    fn get_current() -> Self {
        let ttbr0_el1: u64;
        let ttbr1_el1: u64;
        unsafe {
            asm!("mrs {}, ttbr0_el1", out(reg) ttbr0_el1, options(nomem, nostack, preserves_flags));
            asm!("mrs {}, ttbr1_el1", out(reg) ttbr1_el1, options(nomem, nostack, preserves_flags));
        }
        AddressSpace {
            ttbr0_el1,
            ttbr1_el1,
        }
    }

    fn load(&self) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        unsafe {
            asm!(
                "msr ttbr0_el1, {ttbr0}",
                "msr ttbr1_el1, {ttbr1}",
                "isb",
                ttbr0 = in(reg) self.ttbr0_el1,
                ttbr1 = in(reg) self.ttbr1_el1,
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
        todo!("Walk the AArch64 translation tables looking for a free run of pages.")
    }

    fn find_free_region_large_aligned(
        &mut self,
        _n_large_pages: usize,
        _range: (VirtualAddress, VirtualAddress),
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Walk the translation tables looking for a free run of 2 MiB blocks.")
    }

    fn find_free_region_huge_aligned(
        &mut self,
        _n_huge_pages: usize,
        _range: (VirtualAddress, VirtualAddress),
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Walk the translation tables looking for a free run of 1 GiB blocks.")
    }

    fn map_page(
        &mut self,
        _mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Install a level 3 page descriptor for a 4 KiB page.")
    }

    fn unmap_page(
        &mut self,
        _vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Clear the level 3 page descriptor and return the frame it pointed at.")
    }

    fn map_large_page(
        &mut self,
        _mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Install a level 2 block descriptor for a 2 MiB mapping.")
    }

    fn unmap_large_page(
        &mut self,
        _vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Clear the level 2 block descriptor for a 2 MiB mapping.")
    }

    fn map_huge_page(
        &mut self,
        _mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Install a level 1 block descriptor for a 1 GiB mapping.")
    }

    fn unmap_huge_page(
        &mut self,
        _vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Clear the level 1 block descriptor for a 1 GiB mapping.")
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
        todo!("Report whether a 2 MiB block is present.")
    }

    fn is_mapped_huge_page(
        &mut self,
        _vaddr: VirtualAddress,
    ) -> Result<bool, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        todo!("Report whether a 1 GiB block is present.")
    }

    /// Resolves an address using the hardware's own stage 1 translation.
    ///
    /// `at s1e1r` is the cheapest way to do this on AArch64: the MMU performs the walk and reports
    /// the result in `PAR_EL1`, so there is no table walking to write by hand.
    fn translate_address(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        let par_el1: u64;
        unsafe {
            asm!(
                // Address translation, stage 1, EL1, read permissions.
                "at s1e1r, {vaddr}",
                // Weakly ordered ISA is weakly ordered; the result is not visible without this.
                "isb",
                "mrs {par}, par_el1",
                vaddr = in(reg) <VirtualAddress as Into<usize>>::into(vaddr) as u64,
                par = out(reg) par_el1,
                options(nostack, preserves_flags),
            );
        }

        // PAR_EL1.F is set when the translation faulted.
        if par_el1 & 1 == 1 {
            Err(super::Error::Unmapped)
        } else {
            let page_base = (par_el1 & PAR_EL1_PADDR_MASK) as usize;
            let offset = <VirtualAddress as Into<usize>>::into(vaddr) & (PAGE_SIZE - 1);
            PhysicalAddress::try_from(page_base | offset).map_err(Into::into)
        }
    }
}

/// Output address field of `PAR_EL1` for a successful translation.
const PAR_EL1_PADDR_MASK: u64 = 0x0000_ffff_ffff_f000;
