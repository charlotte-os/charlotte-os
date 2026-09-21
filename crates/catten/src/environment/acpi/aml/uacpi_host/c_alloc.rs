//! Unsized C allocation with metadata kept in the allocation itself.
use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::ffi::c_void;

// All supported 64-bit C ABIs require at most 16-byte fundamental alignment.
// Keeping the header the same size preserves that alignment for the payload.
const MALLOC_ALIGN: usize = 16;
const HEADER_SIZE: usize = 16;

pub extern "C" fn malloc(size: usize) -> *mut c_void {
    let Some(total) = size.max(1).checked_add(HEADER_SIZE) else {
        return core::ptr::null_mut();
    };
    let Ok(layout) = Layout::from_size_align(total, MALLOC_ALIGN) else {
        return core::ptr::null_mut();
    };
    let allocation = unsafe { alloc(layout) };
    if allocation.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        allocation.cast::<usize>().write(total);
        allocation.add(HEADER_SIZE).cast()
    }
}

/// `ptr` must be null or a live allocation returned by `malloc`.
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe {
            let allocation = ptr.cast::<u8>().sub(HEADER_SIZE);
            let total = allocation.cast::<usize>().read();
            dealloc(allocation, Layout::from_size_align_unchecked(total, MALLOC_ALIGN));
        }
    }
}

/// Fallible allocation for opaque host objects. Pair with `free_object`.
pub(super) fn allocate_object<T>(value: T) -> *mut T {
    assert!(core::mem::align_of::<T>() <= MALLOC_ALIGN);
    let ptr = malloc(core::mem::size_of::<T>()).cast::<T>();
    if !ptr.is_null() {
        unsafe { ptr.write(value) };
    }
    ptr
}

/// `ptr` must be null or a live object returned by `allocate_object`.
pub(super) unsafe fn free_object<T>(ptr: *mut T) {
    if !ptr.is_null() {
        unsafe {
            ptr.drop_in_place();
            free(ptr.cast());
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn uacpi_kernel_alloc(size: usize) -> *mut c_void {
    malloc(size)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_free(ptr: *mut c_void) {
    unsafe { free(ptr) };
}
