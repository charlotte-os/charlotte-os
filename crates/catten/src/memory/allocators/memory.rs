use crate::{
    cpu::{
        isa::interface::memory::AddressSpaceInterface,
        multiprocessor::spin::mutex::MutexCore,
    },
    logln,
    memory::{
        AddressSpace,
        KERNEL_AS,
        PHYSICAL_FRAME_ALLOCATOR,
        linear::{
            MemoryMapping,
            PageType,
            VirtualAddress,
        },
        physical::{
            self,
            *,
        },
    },
};

#[derive(Debug)]
pub enum Error {
    PfaError(physical::Error),
    IsaMemoryError(crate::cpu::isa::memory::Error),
}

impl From<physical::Error> for Error {
    fn from(err: physical::Error) -> Self {
        Error::PfaError(err)
    }
}

impl From<crate::cpu::isa::memory::Error> for Error {
    fn from(err: crate::cpu::isa::memory::Error) -> Self {
        Error::IsaMemoryError(err)
    }
}

pub enum PageSize {
    Standard,
    Large,
    Huge,
}

impl PageSize {
    pub const fn num_bytes(&self) -> usize {
        match self {
            PageSize::Standard => <AddressSpace as AddressSpaceInterface>::PAGE_SIZE,
            PageSize::Large => <AddressSpace as AddressSpaceInterface>::LARGE_PAGE_SIZE,
            PageSize::Huge => <AddressSpace as AddressSpaceInterface>::HUGE_PAGE_SIZE,
        }
    }
}

pub fn try_allocate_and_map_range(
    mut kas_lk: lock_api::MutexGuard<'_, MutexCore, AddressSpace>,
    base: VirtualAddress,
    page_size: PageSize,
    num_pages: usize,
) -> Result<(), Error> {
    let mut mapping = MemoryMapping {
        vaddr: VirtualAddress::default(),
        paddr: PhysicalAddress::default(),
        page_type: PageType::KernelData,
    };
    let alloc_func = match page_size {
        PageSize::Standard => PhysicalFrameAllocator::allocate_frame,
        PageSize::Large => PhysicalFrameAllocator::allocate_large_frame,
        PageSize::Huge => PhysicalFrameAllocator::allocate_huge_frame,
    };
    let mapping_func = match page_size {
        PageSize::Standard => <AddressSpace as AddressSpaceInterface>::map_page,
        PageSize::Large => <AddressSpace as AddressSpaceInterface>::map_large_page,
        PageSize::Huge => <AddressSpace as AddressSpaceInterface>::map_huge_page,
    };
    // allocate and map the pages
    // if mapping fails, deallocate and unmap the frames that were allocated
    for page_idx in 0..num_pages {
        let frame = match alloc_func(&mut PHYSICAL_FRAME_ALLOCATOR.lock()) {
            Ok(f) => f,
            Err(err) => {
                // release the lock so the unmap_and_deallocate_range function can acquire it
                drop(kas_lk);
                unmap_and_deallocate_range(base, page_size, page_idx);
                return Err(Error::PfaError(err));
            }
        };
        let vaddr = base + (page_idx * page_size.num_bytes()) as isize;
        mapping.vaddr = vaddr;
        mapping.paddr = frame;
        if let Err(err) = mapping_func(&mut kas_lk, mapping.clone()) {
            // release the lock so the unmap_and_deallocate_range function can acquire it
            drop(kas_lk);
            // deallocate and unmap the frames that were allocated
            unmap_and_deallocate_range(base, page_size, page_idx + 1);
            // deallocate the frame that was just allocated
            if let Err(err) = PHYSICAL_FRAME_ALLOCATOR.lock().deallocate_frame(frame) {
                logln!("Error deallocating frame at {frame:?} during cleanup: {err:?}");
            }
            return Err(Error::IsaMemoryError(err));
        }
    }
    Ok(())
}

pub fn unmap_and_deallocate_range(base: VirtualAddress, page_size: PageSize, num_pages: usize) {
    let mut kas = KERNEL_AS.lock();
    let dealloc_func = match page_size {
        PageSize::Standard => PhysicalFrameAllocator::deallocate_frame,
        PageSize::Large => PhysicalFrameAllocator::deallocate_large_frame,
        PageSize::Huge => PhysicalFrameAllocator::deallocate_huge_frame,
    };
    let unmapping_func = match page_size {
        PageSize::Standard => <AddressSpace as AddressSpaceInterface>::unmap_page,
        PageSize::Large => <AddressSpace as AddressSpaceInterface>::unmap_large_page,
        PageSize::Huge => <AddressSpace as AddressSpaceInterface>::unmap_huge_page,
    };
    for page_idx in 0..num_pages {
        let vaddr = base + (page_idx * page_size.num_bytes()) as isize;
        if let Ok(paddr) = kas.translate_address(vaddr) {
            if let Err(err) = dealloc_func(&mut PHYSICAL_FRAME_ALLOCATOR.lock(), paddr) {
                logln!("Error deallocating frame at {paddr:?} during cleanup: {err:?}");
            }
            if let Err(err) = unmapping_func(&mut kas, vaddr) {
                logln!("Error unmapping vaddr {vaddr:?} during cleanup: {err:?}");
            }
        }
    }
}
