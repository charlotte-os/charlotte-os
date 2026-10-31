use spin::LazyLock;

use crate::cpu::isa::constants::msrs;
use crate::cpu::isa::interface::memory::address::VirtualAddressIfce;
use crate::memory::linear::address_map::{LA_MAP, RegionType};
use crate::memory::{AddressSpaceInterface, KERNEL_AS, PhysicalAddress, VirtualAddress};

pub static XAPIC_IO_BASE: LazyLock<VirtualAddress> = LazyLock::new(map_xapic_io_page);

pub const APIC_BASE_ADDRESS_MASK: u64 = 0xfffff000;

fn map_xapic_io_page() -> VirtualAddress {
    let apic_base_reg_val = unsafe { msrs::read(msrs::APIC_BASE_ADDRESS_REG) };
    let apic_base_address = PhysicalAddress::from(apic_base_reg_val & APIC_BASE_ADDRESS_MASK);
    let mut kas = KERNEL_AS.lock();
    kas.find_and_map_range(
        apic_base_address,
        crate::memory::linear::PageType::Mmio,
        crate::memory::allocators::memory::PageSize::Standard,
        1,
        (*LA_MAP.get_region(RegionType::KernelMmio)).into(),
    )
    .expect("Failed to map the xAPIC MMIO page.")
}

pub unsafe fn xapic_io_write32(offset: u16, value: u32) {
    let io_base = *XAPIC_IO_BASE;
    let addr = io_base + offset as usize;
    unsafe {
        core::ptr::write_volatile(addr.into_mut::<u32>(), value);
    }
}

pub unsafe fn xapic_io_read32(offset: u16) -> u32 {
    let io_base = *XAPIC_IO_BASE;
    let addr = io_base + offset as usize;
    unsafe { core::ptr::read_volatile(addr.into_ptr::<u32>()) }
}
