//! # Page Table Hierarchy Walker
//!
//! This module implements the page table hierarchy walker for the x86_64 architecture.
//! This structure performs the actual page table walk, translating virtual addresses to physical
//! addresses, mapping pages, and unmapping pages as well as adding and removing page table entries
//! and page tables as needed.

use core::ptr::NonNull;

use super::{is_pagetable_unused, PAGE_SIZE};
use crate::cpu::isa::interface::memory::address::VirtualAddress;
use crate::cpu::isa::interface::memory::{AddressSpaceInterface, MemoryInterface};
use crate::cpu::isa::x86_64::memory::address::paddr::PAddr;
use crate::cpu::isa::x86_64::memory::address::vaddr::VAddr;
use crate::memory::PHYSICAL_FRAME_ALLOCATOR;

const CR3_ADDRESS_MASK: u64 = 0x000ffffffffff000;
type WalkerError = <super::MemoryInterfaceImpl as MemoryInterface>::Error;
type WalkerResult<T> = Result<T, WalkerError>;

pub struct PthWalker<'vas> {
    pub address_space: &'vas mut super::AddressSpace,
    pub vaddr: VAddr,
    pub pml4_ptr: *mut super::PageTable,
    pub pdpt_ptr: *mut super::PageTable,
    pub pd_ptr: *mut super::PageTable,
    pub pt_ptr: *mut super::PageTable,
    pub page_frame_ptr: *mut [u8; super::PAGE_SIZE],
}

impl<'vas> PthWalker<'vas> {
    pub fn new(address_space: &'vas mut super::AddressSpace, vaddr: VAddr) -> Self {
        Self {
            address_space,
            vaddr,
            pml4_ptr: core::ptr::null_mut(),
            pdpt_ptr: core::ptr::null_mut(),
            pd_ptr: core::ptr::null_mut(),
            pt_ptr: core::ptr::null_mut(),
            page_frame_ptr: core::ptr::null_mut(),
        }
    }

    fn unmapped_error() -> WalkerError {
        <super::MemoryInterfaceImpl as MemoryInterface>::Error::Unmapped
    }

    fn already_mapped_error() -> WalkerError {
        <super::MemoryInterfaceImpl as MemoryInterface>::Error::AlreadyMapped
    }

    fn root_table_ptr(&self) -> *mut super::PageTable {
        PAddr::try_from((self.address_space.cr3 & CR3_ADDRESS_MASK) as usize).unwrap().into()
    }

    fn walk_next_level(
        &self,
        table_ptr: *mut super::PageTable,
        index: usize,
        page_size_must_be_set: bool,
        page_size_must_be_clear: bool,
    ) -> WalkerResult<*mut super::PageTable> {
        unsafe {
            let pte = &mut (*table_ptr)[index];
            if !pte.is_present()
                || (page_size_must_be_set && !pte.get_page_size())
                || (page_size_must_be_clear && pte.get_page_size())
            {
                return Err(Self::unmapped_error());
            }
            Ok(pte.try_get_frame().unwrap().into())
        }
    }

    fn set_table_entry(
        entry: &mut super::pte::PageTableEntry,
        frame: PAddr,
        writable: bool,
        user_accessible: bool,
        no_execute: bool,
        page_size: bool,
    ) {
        entry
            .set_frame(frame)
            .set_present(true)
            .set_writable(writable)
            .set_user_accessible(user_accessible)
            .set_execute_disabled(no_execute)
            .set_page_size(page_size);
    }

    fn allocate_and_link_table(
        &mut self,
        parent_table_ptr: *mut super::PageTable,
        parent_index: usize,
        writable: bool,
        user_accessible: bool,
        no_execute: bool,
    ) -> *mut super::PageTable {
        let new_table = PHYSICAL_FRAME_ALLOCATOR.lock().allocate_frame().unwrap();
        unsafe {
            Self::set_table_entry(
                &mut (*parent_table_ptr)[parent_index],
                new_table,
                writable,
                user_accessible,
                no_execute,
                false,
            );
            let new_table_ptr: *mut super::PageTable = new_table.into();
            core::ptr::write_bytes(new_table_ptr.cast::<u8>(), 0, PAGE_SIZE);
            new_table_ptr
        }
    }

    fn ensure_pml4(&mut self) -> *mut super::PageTable {
        if self.pml4_ptr.is_null() {
            // Obtain the PML4 table pointer; all address spaces must have a top level page
            // table as they are all required to map the kernel and
            // higher half memory.
            if self.address_space.cr3 & CR3_ADDRESS_MASK == 0 {
                let new_pml4 = PHYSICAL_FRAME_ALLOCATOR.lock().allocate_frame().unwrap();
                self.address_space.cr3 = <PAddr as Into<u64>>::into(new_pml4) & CR3_ADDRESS_MASK;
                self.address_space.load().expect("Error reloading the CR3 register");
            }
            self.pml4_ptr = self.root_table_ptr();
            unsafe {
                core::ptr::write_bytes(self.pml4_ptr.cast::<u8>(), 0, PAGE_SIZE);
            }
        }
        self.pml4_ptr
    }

    fn prepare_map_walk_result(walk_result: WalkerResult<()>) -> WalkerResult<()> {
        match walk_result {
            Ok(_) => Err(Self::already_mapped_error()),
            Err(<super::MemoryInterfaceImpl as MemoryInterface>::Error::Unmapped) => Ok(()),
            Err(other) => Err(other),
        }
    }

    pub fn walk(&mut self) -> WalkerResult<()> {
        self.pml4_ptr = self.root_table_ptr();
        self.pdpt_ptr =
            self.walk_next_level(self.pml4_ptr, self.vaddr.pml4_index(), false, false)?;
        self.pd_ptr = self.walk_next_level(self.pdpt_ptr, self.vaddr.pdpt_index(), false, true)?;
        self.pt_ptr = self.walk_next_level(self.pd_ptr, self.vaddr.pd_index(), false, true)?;
        self.page_frame_ptr = self
            .walk_next_level(self.pt_ptr, self.vaddr.pt_index(), false, false)?
            .cast::<[u8; super::PAGE_SIZE]>();

        Ok(())
    }

    pub fn walk_large_page(&mut self) -> WalkerResult<()> {
        self.pml4_ptr = self.root_table_ptr();
        self.pdpt_ptr =
            self.walk_next_level(self.pml4_ptr, self.vaddr.pml4_index(), false, false)?;
        self.pd_ptr = self.walk_next_level(self.pdpt_ptr, self.vaddr.pdpt_index(), false, false)?;
        self.pt_ptr = core::ptr::null_mut();
        self.page_frame_ptr = self
            .walk_next_level(self.pd_ptr, self.vaddr.pd_index(), true, false)?
            .cast::<[u8; super::PAGE_SIZE]>();

        Ok(())
    }

    pub fn walk_huge_page(&mut self) -> WalkerResult<()> {
        self.pml4_ptr = self.root_table_ptr();
        self.pdpt_ptr =
            self.walk_next_level(self.pml4_ptr, self.vaddr.pml4_index(), false, false)?;
        self.pd_ptr = core::ptr::null_mut();
        self.pt_ptr = core::ptr::null_mut();
        self.page_frame_ptr = self
            .walk_next_level(self.pdpt_ptr, self.vaddr.pdpt_index(), true, false)?
            .cast::<[u8; super::PAGE_SIZE]>();

        Ok(())
    }

    pub fn map_page(
        &mut self,
        frame: PAddr,
        writable: bool,
        user_accessible: bool,
        no_execute: bool,
    ) -> WalkerResult<()> {
        Self::prepare_map_walk_result(self.walk())?;
        self.ensure_pml4();
        if self.pdpt_ptr.is_null() {
            // Allocate a new page table for the PDPT
            self.pdpt_ptr = self.allocate_and_link_table(
                self.pml4_ptr,
                self.vaddr.pml4_index(),
                writable,
                user_accessible,
                no_execute,
            );
        }
        if self.pd_ptr.is_null() {
            // Allocate a new page table for the PD
            let new_pd = PHYSICAL_FRAME_ALLOCATOR.lock().allocate_frame().unwrap();
            crate::early_logln!(
                "[HEAPDBG] mp new_pd={:#x}",
                (<PAddr as Into<usize>>::into(new_pd.clone()))
            );
            unsafe {
                Self::set_table_entry(
                    &mut (*self.pdpt_ptr)[self.vaddr.pdpt_index()],
                    new_pd,
                    writable,
                    user_accessible,
                    no_execute,
                    false,
                );
            }
            self.pd_ptr = new_pd.into();
            unsafe {
                core::ptr::write_bytes(self.pd_ptr.cast::<u8>(), 0, PAGE_SIZE);
            }
        }
        if self.pt_ptr.is_null() {
            let pde = unsafe { &(*self.pd_ptr)[self.vaddr.pd_index()] };
            if pde.is_present() {
                return Err(<super::MemoryInterfaceImpl as MemoryInterface>::Error::AlreadyMapped);
            }
            // Allocate a new page table for the PT
            self.pt_ptr = self.allocate_and_link_table(
                self.pd_ptr,
                self.vaddr.pd_index(),
                writable,
                user_accessible,
                no_execute,
            );
        }
        // Map the page frame
        unsafe {
            Self::set_table_entry(
                &mut (*self.pt_ptr)[self.vaddr.pt_index()],
                frame,
                writable,
                user_accessible,
                no_execute,
                false,
            );
            // for those who may not immediately see it, this is the Rust equivalent of
            // memset being used to clear the newly mapped page
            core::ptr::write_bytes(<PAddr as Into<*mut u8>>::into(frame), 0, PAGE_SIZE);
        }
        self.address_space.load().expect("Failed to reload the address space");
        unsafe {
            // Get rid of any stale TLB entries referring to the linear address space
            // aperture into which the newly allocated page frame has been mapped
            // This works as is in the single LP world but to operate with multiple
            // processors we need a proper TLB shootdown here.
            core::arch::asm!("invlpg [{}]", in(reg) self.vaddr.into_ptr::<u8>());
        }

        Ok(())
    }

    pub fn unmap_page(&mut self) -> WalkerResult<PAddr> {
        match self.walk() {
            Ok(_) => {
                unsafe {
                    // get the return value
                    let paddr = (*self.pt_ptr)[self.vaddr.pt_index()].try_get_frame().unwrap();
                    // deallocate all higher level tables that are now unused
                    let pte = &raw mut (*self.pt_ptr)[self.vaddr.pt_index()];
                    if (*pte).is_present() {
                        // We do not deallocate the page frame here, as it is the responsibility of
                        // the VMM client calling this function to deallocate the frame if they need
                        // to.
                        (*pte).set_present(false);
                    }

                    let pde = &raw mut (*self.pd_ptr)[self.vaddr.pd_index()];
                    if is_pagetable_unused(NonNull::new_unchecked(self.pt_ptr)) {
                        PHYSICAL_FRAME_ALLOCATOR
                            .lock()
                            .deallocate_frame((*pde).try_get_frame().unwrap())
                            .unwrap();
                        (*pde).set_present(false);
                    }

                    let pdpte = &raw mut (*self.pdpt_ptr)[self.vaddr.pdpt_index()];
                    if is_pagetable_unused(NonNull::new_unchecked(self.pd_ptr)) {
                        PHYSICAL_FRAME_ALLOCATOR
                            .lock()
                            .deallocate_frame((*pdpte).try_get_frame().unwrap())
                            .unwrap();
                        (*pdpte).set_present(false);
                    }

                    let pml4e = &raw mut (*self.pml4_ptr)[self.vaddr.pml4_index()];
                    if is_pagetable_unused(NonNull::new_unchecked(self.pdpt_ptr)) {
                        PHYSICAL_FRAME_ALLOCATOR
                            .lock()
                            .deallocate_frame((*pml4e).try_get_frame().unwrap())
                            .unwrap();
                        (*pml4e).set_present(false);
                    }
                    //super::tlb::invalidate_page(self.address_space, self.vaddr);
                    Ok(paddr)
                }
            }
            Err(other) => Err(other),
        }
    }

    pub fn map_large_page(
        &mut self,
        frame: PAddr,
        writable: bool,
        user_accessible: bool,
        no_execute: bool,
    ) -> WalkerResult<()> {
        Self::prepare_map_walk_result(self.walk_large_page())?;
        self.ensure_pml4();
        if self.pdpt_ptr.is_null() {
            // Allocate a new page table for the PDPT
            self.pdpt_ptr = self.allocate_and_link_table(
                self.pml4_ptr,
                self.vaddr.pml4_index(),
                writable,
                user_accessible,
                no_execute,
            );
        }
        if self.pd_ptr.is_null() {
            // Allocate a new page table for the PD
            self.pd_ptr = self.allocate_and_link_table(
                self.pdpt_ptr,
                self.vaddr.pdpt_index(),
                writable,
                user_accessible,
                no_execute,
            );
        }
        unsafe {
            if (*self.pd_ptr)[self.vaddr.pd_index()].is_present() {
                return Err(Self::already_mapped_error());
            }
            // Map the large page frame directly in the Page Directory (PML2) with the PS
            // bit set
            Self::set_table_entry(
                &mut (*self.pd_ptr)[self.vaddr.pd_index()],
                frame,
                writable,
                user_accessible,
                no_execute,
                true,
            );
        }
        Ok(())
    }

    pub fn unmap_large_page(&mut self) -> WalkerResult<PAddr> {
        self.walk_large_page()?;
        unsafe {
            let pde = &raw mut (*self.pd_ptr)[self.vaddr.pd_index()];
            let paddr = (*pde).try_get_frame().unwrap();
            (*pde).set_present(false);
            //super::tlb::invalidate_page(self.address_space, self.vaddr);
            Ok(paddr)
        }
    }

    pub fn map_huge_page(
        &mut self,
        frame: PAddr,
        writable: bool,
        user_accessible: bool,
        no_execute: bool,
    ) -> WalkerResult<()> {
        Self::prepare_map_walk_result(self.walk_huge_page())?;
        self.ensure_pml4();
        if self.pdpt_ptr.is_null() {
            // Allocate a new page table for the PDPT
            self.pdpt_ptr = self.allocate_and_link_table(
                self.pml4_ptr,
                self.vaddr.pml4_index(),
                writable,
                user_accessible,
                no_execute,
            );
        }
        unsafe {
            if (*self.pdpt_ptr)[self.vaddr.pdpt_index()].is_present() {
                return Err(Self::already_mapped_error());
            }
            // Map the huge page frame directly in the Page Directory Pointer Table (PML3)
            // with the PS bit set
            Self::set_table_entry(
                &mut (*self.pdpt_ptr)[self.vaddr.pdpt_index()],
                frame,
                writable,
                user_accessible,
                no_execute,
                true,
            );
        }
        Ok(())
    }

    pub fn unmap_huge_page(&mut self) -> WalkerResult<PAddr> {
        self.walk_huge_page()?;
        unsafe {
            let pdpte = &raw mut (*self.pdpt_ptr)[self.vaddr.pdpt_index()];
            let paddr = (*pdpte).try_get_frame().unwrap();
            (*pdpte).set_present(false);
            //super::tlb::invalidate_page(self.address_space, self.vaddr);
            Ok(paddr)
        }
    }
}
