//! # Physical Memory Management
//!
//! This module is responsible for managing physical memory. It provides an interface for allocating
//! and freeing physical memory frames.

use limine::memmap::MEMMAP_USABLE;
pub use limine::request::MemmapResponse;

pub use crate::cpu::isa::interface::memory::MemoryInterface;
use crate::cpu::isa::interface::memory::address::Address;
pub use crate::cpu::isa::interface::memory::address::PhysicalAddressIfce;
pub use crate::cpu::isa::memory::MemoryInterfaceImpl;
pub use crate::cpu::isa::memory::address::paddr::{PhysicalAddress, PhysicalAddressError};
use crate::early_logln;
use crate::klib::constants::BITS_PER_BYTE;
use crate::klib::size::kibibytes;
use crate::memory::allocators::memory::PageSize;

/// Page frames are 4 KiB in size on all supported architectures.
const PAGE_FRAME_SIZE: usize = kibibytes(4);

/// DEBUG: physical base of the kernel heap's large frame (0 = unset).
pub static HEAP_PHYS_BASE: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);

#[derive(Debug, Clone, Copy)]
pub enum Error {
    UnableToAllocateTrackingStructure,
    MisalignedPhysicalAddress,
    RequestLargerThanTotalMemory,
    OutOfFrames,
    InvalidPAddr,
    InvalidPhysAlignment,
    CannotDeallocateUnallocatedFrame,
    FrameAlreadyInUse,
    NoOp,
    PAddrError(PhysicalAddressError),
}

impl From<PhysicalAddressError> for Error {
    fn from(err: PhysicalAddressError) -> Self {
        Error::PAddrError(err)
    }
}

#[derive(Debug)]
pub struct PhysicalFrameAllocator {
    bitmap_ptr: *mut u8,
    bitmap_len: usize,
    next_free_hint: usize,
}

unsafe impl Send for PhysicalFrameAllocator {}

impl PhysicalFrameAllocator {
    pub fn mark_frame_unavailable(&mut self, frame_addr: PhysicalAddress) -> Result<(), Error> {
        if <PhysicalAddress as Into<usize>>::into(frame_addr) % PAGE_FRAME_SIZE != 0 {
            return Err(Error::MisalignedPhysicalAddress);
        }
        let idx = addr_to_bitmap_index(frame_addr)?;
        let byte_idx = idx.0;
        let bit_idx = idx.1;
        unsafe {
            if self.bitmap_ptr.offset(byte_idx as isize).read_volatile() & (1 << bit_idx) != 0 {
                Err(Error::FrameAlreadyInUse)
            } else {
                // set the bit corresponding to the frame being marked unavailable
                *self.bitmap_ptr.offset(byte_idx as isize) |= 1 << bit_idx;
                return Ok(());
            }
        }
    }

    pub fn allocate_frame(&mut self) -> Result<PhysicalAddress, Error> {
        // Scan from the hint forward, then wrap around if needed
        if let Some(result) = self.scan_for_free_frame(self.next_free_hint, self.bitmap_len) {
            return result;
        }
        if self.next_free_hint > 0 {
            if let Some(result) = self.scan_for_free_frame(0, self.next_free_hint) {
                return result;
            }
        }
        Err(Error::OutOfFrames)
    }

    fn scan_for_free_frame(
        &mut self,
        start: usize,
        end: usize,
    ) -> Option<Result<PhysicalAddress, Error>> {
        for byte_idx in start..end {
            unsafe {
                let curr_byte_ptr = self.bitmap_ptr.add(byte_idx);
                let byte_val = curr_byte_ptr.read_volatile();
                if byte_val != 0xff {
                    let bit_idx = (!byte_val).trailing_zeros() as usize;
                    curr_byte_ptr.write_volatile(byte_val | (1 << bit_idx));
                    self.next_free_hint = byte_idx;
                    let raw_addr = (byte_idx * BITS_PER_BYTE + bit_idx) * PAGE_FRAME_SIZE;
                    let hb = HEAP_PHYS_BASE.load(core::sync::atomic::Ordering::Relaxed);
                    if hb != 0 && raw_addr >= hb && raw_addr < hb + crate::klib::size::mebibytes(2)
                    {
                        crate::early_logln!(
                            "[HEAPDBG] !!! allocate_frame returned {:#x} INSIDE heap [{:#x},+2MiB)",
                            raw_addr,
                            hb
                        );
                    }
                    return Some(PhysicalAddress::try_from(raw_addr).map_err(Error::from));
                }
            }
        }
        None
    }

    #[inline]
    fn is_containing_frame_available(&self, addr: PhysicalAddress) -> Result<bool, Error> {
        let (byte_idx, bit_idx) = addr_to_bitmap_index(addr.prev_aligned_to(PAGE_FRAME_SIZE))?;
        unsafe {
            let byte = self.bitmap_ptr.offset(byte_idx as isize).read_volatile();
            Ok(byte & (1 << bit_idx) == 0)
        }
    }

    #[inline]
    fn mark_frames_unavailable(
        &mut self,
        start_addr: PhysicalAddress,
        nframes: usize,
    ) -> Result<(), Error> {
        for i in 0..nframes {
            self.mark_frame_unavailable(start_addr + (i * PAGE_FRAME_SIZE) as isize)?;
        }
        Ok(())
    }

    pub fn allocate_contiguous(
        &mut self,
        nframes: usize,
        alignment: usize,
    ) -> Result<PhysicalAddress, Error> {
        if nframes == 0 {
            Err(Error::NoOp)
        } else if nframes / BITS_PER_BYTE > self.bitmap_len {
            Err(Error::RequestLargerThanTotalMemory)
        } else if alignment % PAGE_FRAME_SIZE != 0
            || alignment / (PAGE_FRAME_SIZE * BITS_PER_BYTE) > self.bitmap_len
        {
            Err(Error::InvalidPhysAlignment)
        } else {
            let mut start_frame_base = alignment;

            'outer: loop {
                for fb in (start_frame_base..(start_frame_base + nframes * PAGE_FRAME_SIZE))
                    .step_by(PAGE_FRAME_SIZE)
                {
                    if !self.is_containing_frame_available(
                        PhysicalAddress::try_from(fb as usize).unwrap(),
                    )? {
                        start_frame_base += alignment;
                        break;
                    } else if fb == start_frame_base + (nframes - 1) * PAGE_FRAME_SIZE {
                        // found a suitable range
                        break 'outer;
                    }
                }
            }
            let start_addr = PhysicalAddress::try_from(start_frame_base)?;
            self.mark_frames_unavailable(start_addr, nframes)?;
            Ok(start_addr) // Placeholder
        }
    }

    pub fn deallocate_frame(&mut self, frame_addr: PhysicalAddress) -> Result<(), Error> {
        if <PhysicalAddress as Into<usize>>::into(frame_addr.clone()) % PAGE_FRAME_SIZE != 0 {
            return Err(Error::MisalignedPhysicalAddress);
        }
        if let Ok(idx) = addr_to_bitmap_index(frame_addr) {
            let byte_idx = idx.0;
            let bit_idx = idx.1;
            unsafe {
                if self.bitmap_ptr.offset(byte_idx as isize).read_volatile() & (1 << bit_idx) == 0 {
                    Err(Error::CannotDeallocateUnallocatedFrame)
                } else {
                    // clear the bit corresponding to the frame being deallocated
                    *self.bitmap_ptr.offset(byte_idx as isize) &= !(1 << bit_idx);
                    self.next_free_hint = self.next_free_hint.min(byte_idx);
                    return Ok(());
                }
            }
        } else {
            Err(Error::InvalidPAddr)
        }
    }

    pub fn allocate_large_frame(&mut self) -> Result<PhysicalAddress, Error> {
        let r = self.allocate_contiguous(
            PageSize::Large.num_bytes() / PAGE_FRAME_SIZE,
            PageSize::Large.num_bytes(),
        );
        if let Ok(ref frame) = r {
            let raw: usize = <PhysicalAddress as Into<usize>>::into(frame.clone());
            crate::early_logln!("[HEAPDBG] allocate_large_frame -> {:#x}", raw);
            HEAP_PHYS_BASE.store(raw, core::sync::atomic::Ordering::Relaxed);
        }
        r
    }

    pub fn deallocate_large_frame(&mut self, frame_addr: PhysicalAddress) -> Result<(), Error> {
        for i in 0..(PageSize::Large.num_bytes() / PAGE_FRAME_SIZE) {
            self.deallocate_frame(frame_addr + (i * PAGE_FRAME_SIZE) as isize)?;
        }
        Ok(())
    }

    pub fn allocate_huge_frame(&mut self) -> Result<PhysicalAddress, Error> {
        self.allocate_contiguous(
            PageSize::Huge.num_bytes() / PAGE_FRAME_SIZE,
            PageSize::Huge.num_bytes(),
        )
    }

    pub fn deallocate_huge_frame(&mut self, frame_addr: PhysicalAddress) -> Result<(), Error> {
        for i in 0..(PageSize::Huge.num_bytes() / PAGE_FRAME_SIZE) {
            self.deallocate_frame(frame_addr + (i * PAGE_FRAME_SIZE) as isize)?;
        }
        Ok(())
    }

    pub fn allocate_frames(
        &mut self,
        frame_addrs: &mut [PhysicalAddress],
        page_size: PageSize,
    ) -> Result<(), Error> {
        let alloc_fn = match page_size {
            PageSize::Standard => Self::allocate_frame,
            PageSize::Large => Self::allocate_large_frame,
            PageSize::Huge => Self::allocate_huge_frame,
        };
        for i in 0..frame_addrs.len() {
            let frame = alloc_fn(self)?;
            frame_addrs[i] = frame;
        }
        Ok(())
    }

    pub fn deallocate_frames(
        &mut self,
        frame_addrs: &[PhysicalAddress],
        page_size: PageSize,
    ) -> Result<(), Error> {
        let dealloc_fn = match page_size {
            PageSize::Standard => Self::deallocate_frame,
            PageSize::Large => Self::deallocate_large_frame,
            PageSize::Huge => Self::deallocate_huge_frame,
        };
        for frame in frame_addrs {
            dealloc_fn(self, *frame)?;
        }
        Ok(())
    }
}

// There should be a From implementation for each type of memory map we support.

impl From<&MemmapResponse> for PhysicalFrameAllocator {
    fn from(response: &MemmapResponse) -> Self {
        early_logln!("Computing PhysicalFrameAllocator bitmap size...");
        let bitmap_size = compute_bitmap_size(response);
        early_logln!("PhysicalFrameAllocator bitmap size: {:?} bytes", bitmap_size);
        early_logln!("Finding best fit memory location for the PhysicalFrameAllocator bitmap...");
        let bitmap_addr: PhysicalAddress = find_mmap_best_fit(response, bitmap_size).unwrap();
        early_logln!("PhysicalFrameAllocator bitmap addr (physical): {:?}", bitmap_addr);
        let pfa = PhysicalFrameAllocator {
            bitmap_ptr: unsafe { bitmap_addr.into_hhdm_mut::<u8>() },
            bitmap_len: bitmap_size,
            next_free_hint: 0,
        };
        // Initially mark all frames as unavailable.
        early_logln!("Clearing PhysicalFrameAllocator bitmap...");
        for i in 0..bitmap_size {
            unsafe {
                *(pfa.bitmap_ptr.offset(i as isize)) = 0xffu8;
            }
        }
        early_logln!("Initializing PhysicalFrameAllocator bitmap...");
        for entry in response.entries().iter() {
            early_logln!(
                "[HEAPDBG] mmap base={:#x} len={:#x} type={}",
                (entry.base),
                (entry.length),
                (entry.type_)
            );
        }
        init_bitmap_from_mmap(pfa.bitmap_ptr, response);
        //address zero is not accessible
        unsafe {
            *pfa.bitmap_ptr |= 1;
        }
        // Mark the bitmap region as unusable.
        mark_pfa_bitmap_unusable(pfa.bitmap_ptr, bitmap_addr, bitmap_size);
        early_logln!("PhysicalFrameAllocator bitmap initialized.");

        pfa
    }
}

fn compute_bitmap_size(mmap: &MemmapResponse) -> usize {
    let mut highest_address: PhysicalAddress = unsafe { PhysicalAddress::from_unchecked(0usize) };
    // Find the highest address in the memory map.
    for entry in mmap.entries().iter() {
        let entry_end = entry.base + entry.length;
        if entry_end > <PhysicalAddress as Into<usize>>::into(highest_address) as u64 {
            highest_address = unsafe { PhysicalAddress::from_unchecked(entry_end as usize) };
        }
    }
    // Compute the size of the bitmap needed to track all frames up to the highest address.
    let haddr_raw = <PhysicalAddress as Into<usize>>::into(highest_address);
    let num_pages = haddr_raw / PAGE_FRAME_SIZE
        + if haddr_raw % PAGE_FRAME_SIZE > 0 {
            1
        } else {
            0
        };
    let num_bmap_bytes = num_pages / BITS_PER_BYTE
        + if num_pages % BITS_PER_BYTE > 0 {
            1
        } else {
            0
        };
    num_bmap_bytes
}

// Helper functions

fn find_mmap_best_fit(mmap: &MemmapResponse, size: usize) -> Result<PhysicalAddress, Error> {
    let mut best_fit = PhysicalAddress::try_from(0usize)?;
    let mut best_fit_size = 0;
    for entry in mmap.entries().iter() {
        let entry_size = entry.length;
        if entry_size >= size as u64 && (best_fit_size == 0 || entry_size < best_fit_size) {
            best_fit = PhysicalAddress::try_from(entry.base as usize)?;
            best_fit_size = entry_size;
        }
    }
    if best_fit == PhysicalAddress::try_from(0usize)? {
        Err(Error::UnableToAllocateTrackingStructure)
    } else {
        Ok(best_fit)
    }
}

fn addr_to_bitmap_index(addr: PhysicalAddress) -> Result<(usize, usize), Error> {
    if <PhysicalAddress as Into<usize>>::into(addr) % PAGE_FRAME_SIZE != 0 {
        return Err(Error::MisalignedPhysicalAddress);
    }

    let bit_index = <PhysicalAddress as Into<usize>>::into(addr) >> 12; // divide by PAGE_FRAME_SIZE

    let byte_index = bit_index / BITS_PER_BYTE;
    let bit_offset = bit_index % BITS_PER_BYTE;

    Ok((byte_index, bit_offset))
}

fn init_bitmap_from_mmap(bitmap_ptr: *mut u8, mmap: &MemmapResponse) {
    for entry in mmap.entries().iter() {
        if entry.type_ == MEMMAP_USABLE {
            let start = entry.base;
            let end = entry.base + entry.length;
            for i in (start..end).step_by(PAGE_FRAME_SIZE) {
                //logln!("Marking frame at physical address {:?} as available...", i);
                let (byte_index, bit_offset) =
                    addr_to_bitmap_index(PhysicalAddress::try_from(i as usize).unwrap()).unwrap();
                unsafe {
                    *(bitmap_ptr.offset(byte_index as isize)) &= !(1 << bit_offset);
                }
            }
        }
    }
}

fn mark_pfa_bitmap_unusable(bitmap_ptr: *mut u8, base: PhysicalAddress, length: usize) {
    let n_pages = if length % PAGE_FRAME_SIZE > 0 {
        length / PAGE_FRAME_SIZE + 1
    } else {
        length / PAGE_FRAME_SIZE
    };

    for i in 0..n_pages {
        let pfa_index = addr_to_bitmap_index(base + (i * PAGE_FRAME_SIZE) as isize)
            .expect("Failed to convert PAddr to bitmap index.");
        unsafe {
            *(bitmap_ptr.offset(pfa_index.0 as isize)) |= 1 << pfa_index.1;
        }
    }
}
