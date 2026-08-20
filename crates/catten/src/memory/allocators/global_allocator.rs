use core::{
    ptr::NonNull,
    sync::atomic::{
        AtomicPtr,
        Ordering,
    },
};

use talc::{
    base::{
        Talc,
        binning::Binning,
    },
    source::Source,
    *,
};

use crate::{
    cpu::{
        isa::interface::memory::address::VirtualAddressIfce,
        multiprocessor::spin::mutex::MutexCore,
    },
    klib::size::mebibytes,
    memory::{
        KERNEL_AS,
        VirtualAddress,
        allocators::memory::{
            PageSize,
            try_allocate_and_map_range,
        },
        linear::address_map::{
            LA_MAP,
            RegionType::KernelAllocatorArena,
        },
    },
};

const INITIAL_HEAP_SIZE: usize = mebibytes(2);
static ACQUIRE_COUNT: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
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
        let he = returned_ptr.as_ptr();
        let tag_now = he.wrapping_sub(1).read();
        let size_now = (he.wrapping_sub(8) as *const usize).read();
        // also probe a few physical aliases via HHDM and the heap mapping
        let mid = base.into_mut::<u8>().wrapping_add(0x100000);
        mid.write(0xab);
        let mid_read = mid.read();
        crate::early_logln!(
            "[HEAPDBG] claim base={:p} heap_end={:p} tag@-1={:#x} size@-8={:#x} \
             mid_write_read={:#x}",
            (base.into_mut::<u8>()),
            he,
            tag_now,
            size_now,
            mid_read
        );
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
        let n = ACQUIRE_COUNT.fetch_add(1, Ordering::Relaxed);
        let tag_at_end = unsafe { curr_end.wrapping_sub(1).read() };
        let size_at_end = unsafe { (curr_end.wrapping_sub(8) as *const usize).read() };
        let c = talc.counters();
        crate::early_logln!(
            "[HEAPDBG] acquire #{} curr_end={:p} align={:#x} req_size={:#x} tag@-1={:#x} \
             size@-8={:#x} | claimed={:#x} available={:#x} allocated={:#x}",
            n,
            curr_end,
            (layout.align()),
            (layout.size()),
            tag_at_end,
            size_at_end,
            (c.claimed_bytes),
            (c.available_bytes),
            (c.allocated_bytes)
        );
        let new_region_start = VirtualAddress::from(curr_end as usize);
        let new_region_end = new_region_start + PageSize::Large.num_bytes();
        /* Actually allocate and map the new region */
        try_allocate_and_map_range(KERNEL_AS.lock(), new_region_start, PageSize::Large, 1)
            .expect("Failed to allocate and extend the kernel heap");
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
