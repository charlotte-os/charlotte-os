use spin::LazyLock;

use crate::cpu::isa::interface::memory::address::VirtualAddressIfce;
use crate::environment::boot_protocol::limine::RSDP_REQUEST;
use crate::memory::{AddressSpace, AddressSpaceInterface, PhysicalAddress, VirtualAddress};

#[doc = include_str!("doc.md")]
pub mod aml;
pub mod sdt;
pub mod table_map;

pub enum Error {
    IrqValOutOfRange,
}

pub static RSDP_ADDR: LazyLock<PhysicalAddress> = LazyLock::new(|| {
    AddressSpace::get_current()
        .translate_address(VirtualAddress::from_mut(
            RSDP_REQUEST
                .response()
                .expect("Limine failed to provide an physical address for the ACPI RSDP structure.")
                .address,
        ))
        .expect("Unable to obtain the physical address of the ACPI RSDP structure.")
});
