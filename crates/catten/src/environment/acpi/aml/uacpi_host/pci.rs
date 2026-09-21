//! PCI configuration access that is available before device enumeration.
use uacpi_wrapper::*;

use super::c_alloc::{allocate_object, free_object};
use super::memory::{MAP_FAILED, uacpi_kernel_map, uacpi_kernel_unmap};
use crate::environment::acpi::sdt::mcfg::configuration_address;

const ECAM_FUNCTION_SIZE: usize = 4096;

// Each ECAM handle owns its mapping; releasing one cannot invalidate another.
struct PciDevice(*mut u8);

impl PciDevice {
    fn valid_access(&self, offset: usize, width: usize) -> bool {
        offset % width == 0
            && offset.checked_add(width).is_some_and(|end| end <= ECAM_FUNCTION_SIZE)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_pci_device_open(
    address: uacpi_pci_address,
    out_handle: *mut uacpi_handle,
) -> uacpi_status {
    if out_handle.is_null() {
        return UACPI_STATUS_INVALID_ARGUMENT;
    }
    unsafe { out_handle.write(core::ptr::null_mut()) };
    if address.device >= 32 || address.function >= 8 {
        return UACPI_STATUS_INVALID_ARGUMENT;
    }
    let Some(physical) =
        configuration_address(address.segment, address.bus, address.device, address.function)
    else {
        return UACPI_STATUS_NOT_FOUND;
    };
    let mapping = unsafe { uacpi_kernel_map(physical, ECAM_FUNCTION_SIZE) };
    if mapping == MAP_FAILED {
        return UACPI_STATUS_OUT_OF_MEMORY;
    }
    let handle = allocate_object(PciDevice(mapping.cast()));
    if handle.is_null() {
        unsafe { uacpi_kernel_unmap(mapping, ECAM_FUNCTION_SIZE) };
        return UACPI_STATUS_OUT_OF_MEMORY;
    }
    unsafe { out_handle.write(handle.cast()) };
    UACPI_STATUS_OK
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_pci_device_close(handle: uacpi_handle) {
    if handle.is_null() {
        return;
    }
    let device = unsafe { &*handle.cast::<PciDevice>() };
    unsafe { uacpi_kernel_unmap(device.0.cast(), ECAM_FUNCTION_SIZE) };
    unsafe { free_object(handle.cast::<PciDevice>()) };
}

macro_rules! pci_accessors {
    ($read:ident, $write:ident, $ty:ty) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $read(
            handle: uacpi_handle,
            offset: uacpi_size,
            out_value: *mut $ty,
        ) -> uacpi_status {
            if handle.is_null() || out_value.is_null() {
                return UACPI_STATUS_INVALID_ARGUMENT;
            }
            let device = unsafe { &*handle.cast::<PciDevice>() };
            if !device.valid_access(offset, core::mem::size_of::<$ty>()) {
                return UACPI_STATUS_INVALID_ARGUMENT;
            }
            // Offsets are bytes even for 16-bit and 32-bit accesses.
            let value = unsafe { device.0.add(offset).cast::<$ty>().read_volatile().to_le() };
            unsafe { out_value.write(value) };
            UACPI_STATUS_OK
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
            let device = unsafe { &*handle.cast::<PciDevice>() };
            if !device.valid_access(offset, core::mem::size_of::<$ty>()) {
                return UACPI_STATUS_INVALID_ARGUMENT;
            }
            unsafe { device.0.add(offset).cast::<$ty>().write_volatile(in_value.to_le()) };
            UACPI_STATUS_OK
        }
    };
}

pci_accessors!(uacpi_kernel_pci_read8, uacpi_kernel_pci_write8, u8);
pci_accessors!(uacpi_kernel_pci_read16, uacpi_kernel_pci_write16, u16);
pci_accessors!(uacpi_kernel_pci_read32, uacpi_kernel_pci_write32, u32);

pub(super) fn self_test() {
    let mut registers = [0u32; ECAM_FUNCTION_SIZE / 4];
    let mut device = PciDevice(registers.as_mut_ptr().cast());
    let handle = core::ptr::from_mut(&mut device).cast();
    unsafe {
        assert_eq!(uacpi_kernel_pci_write32(handle, 4, 0x12345678), UACPI_STATUS_OK);
        assert_eq!(registers[1], 0x12345678u32.to_le());
        let mut word = 0;
        assert_eq!(uacpi_kernel_pci_read16(handle, 6, &mut word), UACPI_STATUS_OK);
        assert_eq!(word, 0x1234);
        assert_eq!(uacpi_kernel_pci_write16(handle, 4, 0xabcd), UACPI_STATUS_OK);
        let mut dword = 0;
        assert_eq!(uacpi_kernel_pci_read32(handle, 4, &mut dword), UACPI_STATUS_OK);
        assert_eq!(dword, 0x1234abcd);
        assert_eq!(uacpi_kernel_pci_write32(handle, 4092, 0x89abcdef), UACPI_STATUS_OK);
        assert_eq!(registers[1023], 0x89abcdefu32.to_le());
        assert_eq!(uacpi_kernel_pci_read16(handle, 4095, &mut word), UACPI_STATUS_INVALID_ARGUMENT);
        assert_eq!(
            uacpi_kernel_pci_read32(handle, usize::MAX, &mut dword),
            UACPI_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(
            uacpi_kernel_pci_read32(handle, 4096, &mut dword),
            UACPI_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(uacpi_kernel_pci_read32(handle, 1, &mut dword), UACPI_STATUS_INVALID_ARGUMENT);
    }
}
