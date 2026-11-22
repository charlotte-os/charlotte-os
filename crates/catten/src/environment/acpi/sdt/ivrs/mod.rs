//! # I/O Virtualization Reporting Structure (IVRS) Table
//!
//! Parsing and handling of the ACPI IVRS table to determine the presence and capabilities of AMD-Vi
//! IOMMUs.
//!
//! Ref: AMD I/O Virtualization Technology (IOMMU) Specification Revision 3.11
mod device_entry;
mod ivhd;
mod ivmd;

use crate::device_management::drivers::busses::pci_express::topology::PcieLocation;
use crate::klib::bitwise;

pub enum Error {
    InvalidIvdbType,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct IvInfo(u32);

impl IvInfo {
    fn extended_feature_support(self) -> bool {
        self.0 & 1 != 0
    }

    fn preboot_dma_remapping_used(self) -> bool {
        (self.0 >> 1) & 1 != 0
    }

    fn guest_virtual_address_width(self) -> u8 {
        const GVA_SIZE_SHIFT: u8 = 5;
        const GVA_SIZE_MASK: u32 = 0b111;
        bitwise::read_subfield(self.0, GVA_SIZE_SHIFT, GVA_SIZE_MASK) as u8
    }

    fn system_physical_address_width(self) -> u8 {
        const PA_SIZE_SHIFT: u8 = 8;
        const PA_SIZE_MASK: u32 = (1 << 7) - 1;
        bitwise::read_subfield(self.0, PA_SIZE_SHIFT, PA_SIZE_MASK) as u8
    }

    fn system_virtual_address_width(self) -> u8 {
        const VA_SIZE_SHIFT: u8 = 15;
        const VA_SIZE_MASK: u32 = (1 << 7) - 1;
        bitwise::read_subfield(self.0, VA_SIZE_SHIFT, VA_SIZE_MASK) as u8
    }

    fn is_ats_response_address_translation_range_reserved(self) -> bool {
        (self.0 >> 22) & 1 != 0
    }
}

#[repr(C, packed)]
pub struct IvrsHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    _oem_id: [u8; 6],
    _oem_table_id: u64,
    _oem_revision: u32,
    _creator_id: u32,
    _creator_revision: u32,
    iv_info: IvInfo,
    _reserved: u64,
}

const _: () = assert!(size_of::<IvrsHeader>() == 48);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EntryType {
    // 4-byte entries
    Reserved = 0,
    AllDevicesUnderIommu = 1,
    Select = 2,
    StartOfRange = 3,
    EndOfRange = 4,
    // IVHDs
    IvhdLegacy = 16,
    IvhdModern = 17,
    // IVMDs
    IvmdAll = 32,
    IvmdSelect = 33,
    IvmdRange = 34,
    IvhdModernVariable = 64,
    // 8-byte entries
    Reserved2 = 65,
    AliasSelect = 66,
    AliasStartOfRange = 67,
    Reserved4 = 68,
    Reserved5 = 69,
    ExtendedSelect = 70,
    ExtendedStartOfRange = 71,
    SpecialDevice = 72,
    // variable length entry
    AcpiHid = 240,
}

impl TryFrom<u8> for EntryType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Reserved,
            1 => Self::AllDevicesUnderIommu,
            2 => Self::Select,
            3 => Self::StartOfRange,
            4 => Self::EndOfRange,
            16 => Self::IvhdLegacy,
            17 => Self::IvhdModern,
            32 => Self::IvmdAll,
            33 => Self::IvmdSelect,
            34 => Self::IvmdRange,
            64 => Self::IvhdModernVariable,
            65 => Self::Reserved2,
            66 => Self::AliasSelect,
            67 => Self::AliasStartOfRange,
            68 => Self::Reserved4,
            69 => Self::Reserved5,
            70 => Self::ExtendedSelect,
            71 => Self::ExtendedStartOfRange,
            72 => Self::SpecialDevice,
            240 => Self::AcpiHid,
            _ => return Err(Error::InvalidIvdbType),
        })
    }
}
