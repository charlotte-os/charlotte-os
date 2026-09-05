//! ACPI System Description Tables (SDTs)

pub mod fadt;
pub mod madt;
pub mod mcfg;

#[repr(u8)]
pub enum GasAddressSpace {
    SystemMemory = 0,
    SystemIO = 1,
    PciConfigurationSpace = 2,
    EmbeddedController = 3,
    SmBus = 4,
    Cmos = 5,
    PciBarTarget = 6,
    Ipmi = 7,
    Gpio = 8,
    GenericSerialBus = 9,
    PlatformCommunicationsChannel = 10,
}

#[repr(C, packed)]
pub struct GenericAddressStructure {
    address_space: GasAddressSpace,
    bit_width: u8,
    bit_offset: u8,
    access_size: u8, // 8 * pow(2, AccessSize) bytes
    address: u64,
}
