//! SystemIO mappings are views, not exclusive reservations. Firmware frequently
//! maps the same registers through both fixed hardware and operation regions.
use uacpi_wrapper::*;

use super::c_alloc::{allocate_object, free_object};

#[derive(Debug, Clone, Copy)]
struct IoRegion {
    base: u16,
    // usize permits the entire 65536-port address space, including port 0xffff.
    len: usize,
}

impl IoRegion {
    fn new(base: uacpi_io_addr, len: usize) -> Option<Self> {
        let end = base.checked_add(u64::try_from(len).ok()?)?;
        if len == 0 || end > 0x1_0000 {
            return None;
        }
        Some(Self {
            base: base as u16,
            len,
        })
    }

    fn port(&self, offset: usize, width: usize) -> Option<u16> {
        if offset.checked_add(width)? > self.len {
            return None;
        }
        Some(self.base + offset as u16)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_io_map(
    base: uacpi_io_addr,
    len: uacpi_size,
    out_handle: *mut uacpi_handle,
) -> uacpi_status {
    if out_handle.is_null() {
        return UACPI_STATUS_INVALID_ARGUMENT;
    }
    unsafe { out_handle.write(core::ptr::null_mut()) };
    let Some(region) = IoRegion::new(base, len) else {
        return UACPI_STATUS_INVALID_ARGUMENT;
    };
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = region;
        UACPI_STATUS_UNIMPLEMENTED
    }
    #[cfg(target_arch = "x86_64")]
    {
        let handle = allocate_object(region);
        if handle.is_null() {
            return UACPI_STATUS_OUT_OF_MEMORY;
        }
        unsafe { out_handle.write(handle.cast()) };
        UACPI_STATUS_OK
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_io_unmap(handle: uacpi_handle) {
    unsafe { free_object(handle.cast::<IoRegion>()) };
}

macro_rules! io_accessors {
    ($read:ident, $write:ident, $ty:ty, $in:ident, $out:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $read(
            handle: uacpi_handle,
            offset: uacpi_size,
            out_value: *mut $ty,
        ) -> uacpi_status {
            if handle.is_null() || out_value.is_null() {
                return UACPI_STATUS_INVALID_ARGUMENT;
            }
            let region = unsafe { &*handle.cast::<IoRegion>() };
            let Some(port) = region.port(offset, core::mem::size_of::<$ty>()) else {
                return UACPI_STATUS_INVALID_ARGUMENT;
            };
            #[cfg(target_arch = "x86_64")]
            {
                unsafe { out_value.write(crate::cpu::isa::io::$in(port)) };
                UACPI_STATUS_OK
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                let _ = port;
                UACPI_STATUS_UNIMPLEMENTED
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $write(
            handle: uacpi_handle,
            offset: uacpi_size,
            in_value: $ty,
        ) -> uacpi_status {
            if handle.is_null() {
                return UACPI_STATUS_INVALID_ARGUMENT;
            }
            let region = unsafe { &*handle.cast::<IoRegion>() };
            let Some(port) = region.port(offset, core::mem::size_of::<$ty>()) else {
                return UACPI_STATUS_INVALID_ARGUMENT;
            };
            #[cfg(target_arch = "x86_64")]
            {
                crate::cpu::isa::io::$out(port, in_value);
                UACPI_STATUS_OK
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                let _ = (port, in_value);
                UACPI_STATUS_UNIMPLEMENTED
            }
        }
    };
}

io_accessors!(uacpi_kernel_io_read8, uacpi_kernel_io_write8, u8, in8, out8);
io_accessors!(uacpi_kernel_io_read16, uacpi_kernel_io_write16, u16, in16, out16);
io_accessors!(uacpi_kernel_io_read32, uacpi_kernel_io_write32, u32, in32, out32);

pub(super) fn self_test() {
    assert!(IoRegion::new(0x1_0000, 1).is_none());
    assert!(IoRegion::new(u64::MAX, 2).is_none());
    assert!(IoRegion::new(0, 0).is_none());
    assert!(IoRegion::new(0xffff, 2).is_none());
    let full = IoRegion::new(0, 0x1_0000).unwrap();
    assert_eq!(full.port(0xffff, 1), Some(0xffff));
    assert_eq!(full.port(0xffff, 2), None);
    assert_eq!(full.port(usize::MAX, 4), None);
    let region = IoRegion::new(0xfffc, 4).unwrap();
    assert_eq!(region.port(0, 4), Some(0xfffc));
    assert_eq!(region.port(1, 4), None);
}
