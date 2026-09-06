//! # Kernel Stack Allocator
//!
//! This module provides a an allocator for kernel thread stacks.
//!
//! Please note that on all supported architectures, the stack grows towards lower addresses so this
//! is the highest address of the stack. Also be aware that stacks allocated using this allocator
//! are mapped into the kernel stack arena in the higher half which means it is only suitable for
//! allocating stacks for kernel threads. Stacks are surrounded on both sides by guard pages to
//! allow for safe stack overflow detection and when enabled for the owning thread, transparent
//! reallocation such that from that thread's perspective it is as if the stack overflow never
//! happened.

use alloc::vec::Vec;

use hashbrown::HashSet;
use spin::{LazyLock, RwLock};

use super::memory;
use crate::cpu::isa::interface::memory::address::{Address, VirtualAddressIfce};
use crate::cpu::isa::memory::paging::PAGE_SIZE;
use crate::cpu::isa::memory::{MemoryInterface, MemoryInterfaceImpl};
use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::klib::size;
use crate::memory::allocators::memory::{PageSize, try_allocate_and_map_range};
use crate::memory::allocators::{self};
use crate::memory::linear::VirtualAddress;
use crate::memory::linear::address_map::RegionType::KernelStackArena;
use crate::memory::linear::address_map::{LA_MAP, LinearMemoryRegion};
use crate::memory::{AddressSpaceInterface, KERNEL_AS};

pub static KERNEL_GUARD_PAGE_SET: LazyLock<RwLock<HashSet<VirtualAddress>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));
pub static KERNEL_STACK_ALLOCATOR: LazyLock<StackAllocator> = LazyLock::new(|| StackAllocator {
    kernel_stack_cache: Mutex::new(Vec::new()),
});

#[derive(Debug)]
pub enum Error {
    IsaMemoryIfce(<MemoryInterfaceImpl as MemoryInterface>::Error),
    AllocatorsMemory(memory::Error),
    InvalidStack,
    WouldOverflow,
    WouldExceedStackArena,
}

impl From<<MemoryInterfaceImpl as MemoryInterface>::Error> for Error {
    fn from(err: <MemoryInterfaceImpl as MemoryInterface>::Error) -> Self {
        Error::IsaMemoryIfce(err)
    }
}

impl From<allocators::memory::Error> for Error {
    fn from(err: allocators::memory::Error) -> Self {
        Error::AllocatorsMemory(err)
    }
}

#[derive(Debug)]
pub struct StackBuf {
    raw_buf_start: VirtualAddress,
    size: usize,
    pub curr_sp: VirtualAddress,
}

impl StackBuf {
    pub fn new(size: usize) -> Result<Self, Error> {
        let new_buf = KERNEL_STACK_ALLOCATOR.allocate_stack(size)?;

        let mut gp_wlock = KERNEL_GUARD_PAGE_SET.write();
        gp_wlock.insert(new_buf.raw_buf_start);
        gp_wlock.insert(new_buf.raw_buf_start + size - PageSize::Standard.num_bytes());

        Ok(new_buf)
    }

    fn synthesize(raw_buf_start: VirtualAddress, size: usize) -> Self {
        // Insert the guard pages into the global set of guard pages so that we can check for stack
        // overflows in the page fault handler.
        Self {
            raw_buf_start,
            size,
            curr_sp: raw_buf_start + size - PageSize::Standard.num_bytes(),
        }
    }

    pub fn initial_sp(&self) -> VirtualAddress {
        self.raw_buf_start + self.size - PageSize::Standard.num_bytes()
    }

    pub fn align_sp_to(&mut self, align: usize) -> Result<&mut Self, Error> {
        if self.curr_sp.is_aligned_to(align) {
            Ok(self)
        } else {
            let aligned_sp = self.curr_sp.prev_aligned_to(align);
            if aligned_sp < self.raw_buf_start + PageSize::Standard.num_bytes() {
                Err(Error::WouldOverflow)
            } else {
                self.curr_sp = aligned_sp;
                Ok(self)
            }
        }
    }

    pub fn push<T>(&mut self, val: T) -> Result<&mut Self, Error> {
        let new_sp =
            (self.curr_sp - core::mem::size_of::<T>()).prev_aligned_to(core::mem::align_of::<T>());
        if new_sp < self.raw_buf_start + PageSize::Standard.num_bytes() {
            Err(Error::WouldOverflow)
        } else {
            self.curr_sp = new_sp;
            let ptr = self.curr_sp.into_mut::<T>();
            unsafe { ptr.write(val) };
            Ok(self)
        }
    }

    // Safety: This should only be used for dropping instances of StackBuf that are being
    // deallocated. Using it for anything else can cause data corruption since multiple thread could
    // end up using the same stack buffer at the same time.
    unsafe fn shallow_clone(&self) -> Self {
        Self {
            raw_buf_start: self.raw_buf_start,
            size: self.size,
            curr_sp: self.initial_sp(),
        }
    }
}

impl Drop for StackBuf {
    fn drop(&mut self) {
        let mut gp_wlock = KERNEL_GUARD_PAGE_SET.write();
        gp_wlock.remove(&self.raw_buf_start);
        gp_wlock.remove(&(self.raw_buf_start + self.size - PageSize::Standard.num_bytes()));
        KERNEL_STACK_ALLOCATOR.deallocate_stack(unsafe { self.shallow_clone() });
    }
}

pub struct StackAllocator {
    kernel_stack_cache: Mutex<Vec<StackBuf>>,
}

impl StackAllocator {
    fn allocate_stack(&self, size: usize) -> Result<StackBuf, Error> {
        let mut cache_lk = self.kernel_stack_cache.lock();
        if let Some(sb_idx) = cache_lk
            .iter()
            .enumerate()
            .filter(|(_idx, buf)| buf.size >= size)
            .min_by_key(|(_idx, buf)| buf.size)
            .and_then(|(idx, _buf)| Some(idx))
        {
            let stack_buf = cache_lk.remove(sb_idx);
            Ok(stack_buf)
        } else {
            // yeet the lock since this branch doesn't use it
            drop(cache_lk);
            let mut kas_lk = KERNEL_AS.lock();
            let base = kas_lk.find_free_region(
                (size / PAGE_SIZE) + 1,
                <LinearMemoryRegion as Into<(VirtualAddress, VirtualAddress)>>::into(
                    *LA_MAP.get_region(KernelStackArena),
                ),
            )?;
            try_allocate_and_map_range(
                kas_lk,
                base + PageSize::Standard.num_bytes(),
                PageSize::Standard,
                (size / PageSize::Standard.num_bytes()) + 1,
            )?;
            Ok(StackBuf::synthesize(base, size))
        }
    }

    #[inline]
    fn deallocate_stack(&self, stack_buf: StackBuf) {
        self.kernel_stack_cache.lock().push(stack_buf);
    }
}
