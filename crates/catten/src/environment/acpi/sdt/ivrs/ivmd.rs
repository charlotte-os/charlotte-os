use crate::device_management::drivers::busses::pci_express::topology::PcieLocation;
use crate::environment::acpi::sdt::ivrs::EntryType;
use crate::memory::PhysicalAddress;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct IvmdFlags(u8);

impl IvmdFlags {
    pub fn must_be_identity_mapped(self) -> bool {
        if self.is_exclusion_range() {
            false
        } else {
            self.0 & 1 != 0
        }
    }

    pub fn is_readable(self) -> bool {
        if self.is_exclusion_range() {
            false
        } else {
            self.0 & (1 << 1) != 0
        }
    }

    pub fn is_writable(self) -> bool {
        if self.is_exclusion_range() {
            false
        } else {
            self.0 & (1 << 2) != 0
        }
    }

    pub fn is_exclusion_range(self) -> bool {
        self.0 & (1 << 3) != 0
    }
}

/// IVMD block (Types 20h-22h)
///
/// Table 112 of the spec lists an 8-byte Reserved field at offset 8 that overlaps the PCI Segment
/// Group field. The layout in Table 111 is the correct one: bytes 8-9 hold the PCI Segment Group
/// and only bytes 10-15 are reserved.
#[repr(C, packed)]
pub struct Ivmd {
    entry_type: u8,
    pub flags: IvmdFlags,
    length: u16,
    device_id: u16,
    /// End of range device ID. Only applies to table type 22h.
    eor_device_id: u16,
    segment_group_num: u16,
    _reserved: [u8; 6],
    start_address: PhysicalAddress,
    memory_block_length: u64,
}

const _: () = assert!(size_of::<Ivmd>() == 32);

impl Ivmd {
    pub fn entry_type(&self) -> Option<EntryType> {
        EntryType::try_from(self.entry_type).ok()
    }

    pub fn start_device_location(&self) -> PcieLocation {
        PcieLocation {
            segment_group_num: self.segment_group_num,
            bus_segment_num: (self.device_id >> 8) as u8,
            device_num: ((self.device_id >> 3) & 0x1f) as u8,
            function_num: (self.device_id & 0x7) as u8,
        }
    }
}
