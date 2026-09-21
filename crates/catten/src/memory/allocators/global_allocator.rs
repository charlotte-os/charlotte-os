use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};

use talc::base::Talc;
use talc::base::binning::Binning;
use talc::source::Source;
use talc::*;

use crate::cpu::isa::interface::memory::address::VirtualAddressIfce;
use crate::cpu::multiprocessor::spin::mutex::MutexCore;
use crate::klib::size::mebibytes;
use crate::memory::allocators::memory::{PageSize, try_allocate_and_map_range};
use crate::memory::linear::address_map::LA_MAP;
use crate::memory::linear::address_map::RegionType::KernelAllocatorArena;
use crate::memory::{KERNEL_AS, VirtualAddress};

const INITIAL_HEAP_SIZE: usize = mebibytes(2);
#[global_allocator]
pub static PRIMARY_ALLOCATOR: TalcLock<MutexCore, ExtendOnOom> = TalcLock::new(ExtendOnOom::new());

pub fn init_primary_allocator() {
    let base = LA_MAP.get_region(KernelAllocatorArena).base;
    try_allocate_and_map_range(
        KERNEL_AS.lock(),
        base,
        PageSize::Large,
        INITIAL_HEAP_SIZE / PageSize::Large.num_bytes(),
    )
    .expect("Failed to allocate and map initial kernel heap memory");
    unsafe {
        let mut pa_lock = PRIMARY_ALLOCATOR.lock();
        let returned_ptr = pa_lock
            .claim(base.into_mut(), INITIAL_HEAP_SIZE)
            .expect("Talc failed to claim the initial kernel heap");
        pa_lock.source.heap_ptr.store(returned_ptr.as_ptr(), Ordering::Release);
    }
}

#[derive(Debug)]
pub struct ExtendOnOom {
    heap_ptr: AtomicPtr<u8>,
}

unsafe impl Sync for ExtendOnOom {}
unsafe impl Send for ExtendOnOom {}

impl ExtendOnOom {
    const fn new() -> Self {
        ExtendOnOom {
            heap_ptr: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
}

unsafe impl Source for ExtendOnOom {
    fn acquire<B: Binning>(
        talc: &mut Talc<Self, B>,
        _layout: core::alloc::Layout,
    ) -> Result<(), ()> {
        let curr_end = talc.source.heap_ptr.load(Ordering::Acquire);
        let new_region_start = VirtualAddress::from(curr_end as usize);
        let new_region_end = new_region_start + PageSize::Large.num_bytes();
        /* Actually allocate and map the new region */
        try_allocate_and_map_range(KERNEL_AS.lock(), new_region_start, PageSize::Large, 1)
            .map_err(|_| ())?;
        unsafe {
            talc.extend(
                NonNull::new(curr_end).expect("Passed null pointer to the constructor of NonNull"),
                new_region_end.into_mut(),
            );
        }
        talc.source.heap_ptr.store(new_region_end.into_mut(), Ordering::Release);
        Ok(())
    }
}
