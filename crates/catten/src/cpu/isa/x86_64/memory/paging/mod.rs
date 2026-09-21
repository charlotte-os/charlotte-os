pub mod pte;
pub mod pth_walker;
use core::arch::asm;
use core::iter::Iterator;
use core::ptr::NonNull;

use super::MemoryInterfaceImpl;
use super::address::vaddr::VirtualAddress;
use crate::cpu::isa::interface::memory::address::{Address, VirtualAddressIfce};
use crate::cpu::isa::interface::memory::{AddressSpaceInterface, MemoryInterface, MemoryMapping};
use crate::klib::size::{gibibytes, kibibytes, mebibytes};
use crate::logln;
use crate::memory::PhysicalAddress;
use crate::memory::linear::PageType;

/// Use the bootloader's PAT layout instead of assuming a particular WC/UC index.
/// Leaf entries select device memory as UC; ordinary RAM stays write-back.
fn pat_index(page_type: PageType) -> u8 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!("rdmsr", in("ecx") 0x277u32, out("eax") low, out("edx") high, options(nomem, nostack));
    }
    let pat = (u64::from(high) << 32) | u64::from(low);
    let desired = if page_type.should_combine_writes() {
        1
    } else if page_type.is_uncacheable() {
        0
    } else {
        6
    };
    let find = |kind| (0..8).find(|index| (pat >> (index * 8)) & 0xff == kind);
    // WC is optional: a framebuffer may safely fall back to UC.
    find(desired)
        .or_else(|| {
            if desired == 1 {
                find(0)
            } else {
                None
            }
        })
        .expect("The CPU PAT lacks the required memory type") as u8
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct HwAsid(u16);

impl HwAsid {
    pub unsafe fn get_inner_unchecked(&self) -> u16 {
        self.0
    }

    pub extern "C" fn get_inner(&self) -> u16 {
        self.0 & 0xfff
    }
}

impl TryFrom<u16> for HwAsid {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value & !0xfff == 0 {
            Ok(HwAsid(value))
        } else {
            Err(())
        }
    }
}

pub const PAGE_SIZE: usize = 4096;
pub const N_PAGE_TABLE_ENTRIES: usize = 512;
pub type PageTable = [pte::PageTableEntry; N_PAGE_TABLE_ENTRIES];

pub fn is_pagetable_unused(table_ptr: NonNull<PageTable>) -> bool {
    unsafe {
        for i in 0..N_PAGE_TABLE_ENTRIES {
            if (table_ptr.as_ref())[i].is_present() {
                return false;
            }
        }
    }
    true
}

#[repr(transparent)]
pub struct AddressSpace {
    // control register 3 i.e. top level page table base register
    cr3: u64,
}

impl AddressSpace {
    /// Retire an ACPI leaf without recycling paging structures still cached on
    /// other processors. The caller must never reuse this virtual address.
    pub fn unmap_page_retaining_tables(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<PhysicalAddress, super::Error> {
        let mut walker = pth_walker::PthWalker::new(self, vaddr);
        walker.walk()?;
        unsafe {
            let entry = &mut (*walker.pt_ptr)[vaddr.pt_index()];
            let frame = entry.try_get_frame()?;
            entry.set_present(false);
            asm!("invlpg [{}]", in(reg) vaddr.into_ptr::<u8>(), options(nostack, preserves_flags));
            Ok(frame)
        }
    }

    pub fn get_cr3(&self) -> u64 {
        self.cr3
    }
}

impl AddressSpaceInterface for AddressSpace {
    const HUGE_PAGE_SIZE: usize = gibibytes(1);
    const LARGE_PAGE_SIZE: usize = mebibytes(2);
    const PAGE_SIZE: usize = kibibytes(4);

    fn get_current() -> Self {
        let cr3: u64;
        unsafe {
            asm!("mov {}, cr3", out(reg) cr3);
        }
        AddressSpace {
            cr3: cr3,
        }
    }

    fn load(&self) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        unsafe {
            // Set the top level page table base register
            asm!("mov cr3, {}", in(reg) self.cr3);
        }
        Ok(())
    }

    fn find_free_region(
        &mut self,
        n_pages: usize,
        range: (VirtualAddress, VirtualAddress),
    ) -> Result<
        <MemoryInterfaceImpl as MemoryInterface>::VAddr,
        <MemoryInterfaceImpl as MemoryInterface>::Error,
    > {
        logln!("Finding free region of {} pages in range {:?}...", n_pages, range);
        let mut page_iter = (range.0..range.1).step_by(PAGE_SIZE);
        while let Some(base) = page_iter.next() {
            //logln!("Checking base address: {:?}", base);
            for nth_page in 0..n_pages {
                let curr_page = base + ((nth_page * PAGE_SIZE) as isize);
                //logln!("Checking page: {:?}", curr_page);
                if range.1 - curr_page < (n_pages * PAGE_SIZE) as isize {
                    return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NoRequestedVAddrRegionAvailable);
                }
                if self.is_mapped(curr_page)? {
                    match page_iter.advance_by(nth_page) {
                        Ok(_) => {
                            //logln!("Page {:?} is already mapped, skipping to next base address.", curr_page);
                            break;
                        }
                        Err(_) => return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NoRequestedVAddrRegionAvailable),
                    }
                }
                if nth_page == n_pages - 1 {
                    logln!("Found free region starting at: {:?}", base);
                    return Ok(base);
                }
            }
        }
        Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NoRequestedVAddrRegionAvailable)
    }

    fn find_free_region_large_aligned(
        &mut self,
        n_large_pages: usize,
        range: (VirtualAddress, VirtualAddress),
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        if range.0.is_aligned_to(Self::LARGE_PAGE_SIZE) == false
            || range.1.is_aligned_to(Self::LARGE_PAGE_SIZE) == false
        {
            return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::VAddrNotLargePageAligned);
        }
        logln!("Finding free region of {} large pages in range {:?}...", n_large_pages, range);
        let mut page_iter = (range.0..range.1).step_by(Self::LARGE_PAGE_SIZE);
        while let Some(base) = page_iter.next() {
            //logln!("Checking base address: {:?}", base);
            for nth_page in 0..n_large_pages {
                let curr_page = base + ((nth_page * Self::LARGE_PAGE_SIZE) as isize);
                //logln!("Checking large page: {:?}", curr_page);
                if range.1 - curr_page < (n_large_pages * Self::LARGE_PAGE_SIZE) as isize {
                    return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NoRequestedVAddrRegionAvailable);
                }
                if self.is_mapped_large_page(curr_page)? {
                    match page_iter.advance_by(nth_page) {
                        Ok(_) => {
                            //logln!("Large page {:?} is already mapped, skipping to next base address.", curr_page);
                            break;
                        }
                        Err(_) => return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NoRequestedVAddrRegionAvailable),
                    }
                }
                if nth_page == n_large_pages - 1 {
                    logln!("Found free region starting at: {:?}", base);
                    return Ok(base);
                }
            }
        }
        Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NoRequestedVAddrRegionAvailable)
    }

    fn find_free_region_huge_aligned(
        &mut self,
        n_huge_pages: usize,
        range: (VirtualAddress, VirtualAddress),
    ) -> Result<VirtualAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        if range.0.is_aligned_to(Self::HUGE_PAGE_SIZE) == false
            || range.1.is_aligned_to(Self::HUGE_PAGE_SIZE) == false
        {
            return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::VAddrNotHugePageAligned);
        }
        logln!("Finding free region of {} huge pages in range {:?}...", n_huge_pages, range);
        let mut page_iter = (range.0..range.1).step_by(Self::HUGE_PAGE_SIZE);
        while let Some(base) = page_iter.next() {
            //logln!("Checking base address: {:?}", base);
            for nth_page in 0..n_huge_pages {
                let curr_page = base + ((nth_page * Self::HUGE_PAGE_SIZE) as isize);
                //logln!("Checking huge page: {:?}", curr_page);
                if range.1 - curr_page < (n_huge_pages * Self::HUGE_PAGE_SIZE) as isize {
                    return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NoRequestedVAddrRegionAvailable);
                }
                if self.is_mapped_huge_page(curr_page)? {
                    match page_iter.advance_by(nth_page) {
                        Ok(_) => {
                            //logln!("Huge page {:?} is already mapped, skipping to next base address.", curr_page);
                            break;
                        }
                        Err(_) => return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NoRequestedVAddrRegionAvailable),
                    }
                }
                if nth_page == n_huge_pages - 1 {
                    logln!("Found free region starting at: {:?}", base);
                    return Ok(base);
                }
            }
        }
        Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NoRequestedVAddrRegionAvailable)
    }

    fn map_page(
        &mut self,
        mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        let mut walker = pth_walker::PthWalker::new(self, mapping.vaddr);
        walker.map_page(
            mapping.paddr,
            mapping.page_type.is_writable(),
            mapping.page_type.is_user_accessible(),
            mapping.page_type.is_no_execute(),
            pat_index(mapping.page_type),
        )?;
        Ok(())
    }

    fn unmap_page(
        &mut self,
        vaddr: <MemoryInterfaceImpl as MemoryInterface>::VAddr,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        if <VirtualAddress as Into<usize>>::into(vaddr) == 0 {
            return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NullVAddrNotAllowed);
        }
        if vaddr.page_offset() != 0 {
            return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::VAddrNotPageAligned);
        }
        let mut walker = pth_walker::PthWalker::new(self, vaddr);
        walker.unmap_page()
    }

    fn map_large_page(
        &mut self,
        mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        let mut walker = pth_walker::PthWalker::new(self, mapping.vaddr);
        walker.map_large_page(
            mapping.paddr,
            mapping.page_type.is_writable(),
            mapping.page_type.is_user_accessible(),
            mapping.page_type.is_no_execute(),
            pat_index(mapping.page_type),
        )?;
        Ok(())
    }

    fn unmap_large_page(
        &mut self,
        vaddr: <MemoryInterfaceImpl as MemoryInterface>::VAddr,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        if <VirtualAddress as Into<usize>>::into(vaddr) == 0 {
            return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NullVAddrNotAllowed);
        }
        if !vaddr.is_aligned_to(Self::LARGE_PAGE_SIZE) {
            return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::VAddrNotLargePageAligned);
        }
        let mut walker = pth_walker::PthWalker::new(self, vaddr);
        walker.unmap_large_page()
    }

    fn map_huge_page(
        &mut self,
        mapping: MemoryMapping,
    ) -> Result<(), <MemoryInterfaceImpl as MemoryInterface>::Error> {
        let mut walker = pth_walker::PthWalker::new(self, mapping.vaddr);
        walker.map_huge_page(
            mapping.paddr,
            mapping.page_type.is_writable(),
            mapping.page_type.is_user_accessible(),
            mapping.page_type.is_no_execute(),
            pat_index(mapping.page_type),
        )?;
        Ok(())
    }

    fn unmap_huge_page(
        &mut self,
        vaddr: <MemoryInterfaceImpl as MemoryInterface>::VAddr,
    ) -> Result<PhysicalAddress, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        if <VirtualAddress as Into<usize>>::into(vaddr) == 0 {
            return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::NullVAddrNotAllowed);
        }
        if !vaddr.is_aligned_to(Self::HUGE_PAGE_SIZE) {
            return Err(<MemoryInterfaceImpl as MemoryInterface>::Error::VAddrNotHugePageAligned);
        }
        let mut walker = pth_walker::PthWalker::new(self, vaddr);
        walker.unmap_huge_page()
    }

    fn is_mapped(
        &mut self,
        vaddr: <MemoryInterfaceImpl as MemoryInterface>::VAddr,
    ) -> Result<bool, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        let mut walker = pth_walker::PthWalker::new(self, vaddr);
        match walker.walk() {
            Ok(_) => Ok(true),
            Err(<MemoryInterfaceImpl as MemoryInterface>::Error::Unmapped) => {
                match walker.walk_large_page() {
                    Ok(_) => Ok(true),
                    Err(<MemoryInterfaceImpl as MemoryInterface>::Error::Unmapped) => {
                        match walker.walk_huge_page() {
                            Ok(_) => Ok(true),
                            Err(<MemoryInterfaceImpl as MemoryInterface>::Error::Unmapped) => {
                                Ok(false)
                            }
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    fn is_mapped_large_page(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<bool, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        let mut walker = pth_walker::PthWalker::new(self, vaddr);
        match walker.walk_large_page() {
            Ok(_) => Ok(true),
            Err(<MemoryInterfaceImpl as MemoryInterface>::Error::Unmapped) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn is_mapped_huge_page(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<bool, <MemoryInterfaceImpl as MemoryInterface>::Error> {
        let mut walker = pth_walker::PthWalker::new(self, vaddr);
        match walker.walk_huge_page() {
            Ok(_) => Ok(true),
            Err(<MemoryInterfaceImpl as MemoryInterface>::Error::Unmapped) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn translate_address(
        &mut self,
        vaddr: super::address::vaddr::VirtualAddress,
    ) -> Result<
        super::address::paddr::PhysicalAddress,
        <MemoryInterfaceImpl as MemoryInterface>::Error,
    > {
        let mut walker = pth_walker::PthWalker::new(self, vaddr);
        match walker.walk() {
            Ok(_) => {
                let mut paddr = unsafe { (*(walker.pt_ptr))[vaddr.pt_index()].try_get_frame()? };
                paddr += vaddr.page_offset();
                Ok(paddr)
            }
            Err(<MemoryInterfaceImpl as MemoryInterface>::Error::Unmapped) => {
                match walker.walk_large_page() {
                    Ok(_) => {
                        let mut paddr =
                            unsafe { (*(walker.pd_ptr))[vaddr.pd_index()].try_get_frame()? };
                        paddr += vaddr.page_offset() + (vaddr.pt_index() * PAGE_SIZE);
                        Ok(paddr)
                    }
                    Err(<MemoryInterfaceImpl as MemoryInterface>::Error::Unmapped) => {
                        walker.walk_huge_page()?;
                        let mut paddr =
                            unsafe { (*(walker.pdpt_ptr))[vaddr.pdpt_index()].try_get_frame()? };
                        paddr += vaddr.page_offset()
                            + (vaddr.pt_index() * PAGE_SIZE)
                            + (vaddr.pd_index() * Self::LARGE_PAGE_SIZE);
                        Ok(paddr)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}
