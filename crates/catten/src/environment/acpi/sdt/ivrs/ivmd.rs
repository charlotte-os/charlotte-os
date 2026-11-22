use crate::device_management::drivers::busses::pci_express::topology::PcieLocation;
use crate::environment::acpi::sdt::ivrs::EntryType;
use crate::memory::PhysicalAddress;

pub struct IvmdFlags(u8);

impl IvmdFlags {
    pub fn must_be_identity_mapped(&self) -> bool {
        if self.is_exclusion_range() {
            false
        } else {
            self.0 & 1 != 0
        }
    }

    pub fn is_readable(&self) -> bool {
        if self.is_exclusion_range() {
            false
        } else {
            self.0 & (1 << 1) != 0
        }
    }

    pub fn is_writable(&self) -> bool {
        if self.is_exclusion_range() {
            false
        } else {
            self.0 & (1 << 2) != 0
        }
    }

    pub fn is_exclusion_range(&self) -> bool {
        self.0 & (1 << 3) != 0
    }
}

#[repr(C)]
pub struct Ivmd {
    pub table_type: EntryType,
    pub flags: u8,
    length: u16,
    device_id: u16,
    /// End of range device ID. Only applies to table type 22h.
    eor_device_id: u16,
    segment_group_num: u16,
    _reserved: [u8; 6],
    start_address: PhysicalAddress,
    memory_block_length: usize,
}

impl Ivmd {
    pub fn start_device_location(&self) -> PcieLocation {
        PcieLocation {
            segment_group_num: self.segment_group_num,
            bus_segment_num: (self.device_id >> 8) as u8,
            device_num: ((self.device_id >> 3) & 0x1f) as u8,
            function_num: (self.device_id & 0x7) as u8,
        }
    }
}
