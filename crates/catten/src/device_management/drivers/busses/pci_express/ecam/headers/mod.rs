use crate::device_management::drivers::busses::pci_express;

const PCI_CFG_SPACE_SIZE: u16 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PcieCfgOffset(u16);

impl PcieCfgOffset {
    pub const fn new_const(offset: u16) -> Self {
        assert!(offset < PCI_CFG_SPACE_SIZE);
        Self(offset)
    }

    pub const fn raw(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for PcieCfgOffset {
    type Error = pci_express::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value < PCI_CFG_SPACE_SIZE {
            Ok(Self(value))
        } else {
            Err(pci_express::Error::CfgOffsetOutOfRange)
        }
    }
}

impl From<u8> for PcieCfgOffset {
    fn from(value: u8) -> Self {
        Self(value as u16)
    }
}

impl<T: Into<u16>> core::ops::Add<T> for PcieCfgOffset {
    type Output = PcieCfgOffset;

    fn add(self, rhs: T) -> Self::Output {
        let rhs: u16 = rhs.into();
        let new_offset = self.0 + rhs;
        if new_offset < PCI_CFG_SPACE_SIZE {
            PcieCfgOffset(new_offset)
        } else {
            panic!("CfgOffset overflow");
        }
    }
}

impl Into<u16> for PcieCfgOffset {
    fn into(self) -> u16 {
        self.0
    }
}

impl Into<usize> for PcieCfgOffset {
    fn into(self) -> usize {
        self.0 as usize
    }
}
pub enum CfgRegType {
    U8 = 1,
    U16 = 2,
    U32 = 4,
}

pub struct CfgRegDesc {
    pub offset: PcieCfgOffset,
    pub reg_type: CfgRegType,
}

impl CfgRegDesc {
    pub const fn new(offset: PcieCfgOffset, reg_type: CfgRegType) -> Self {
        CfgRegDesc {
            offset,
            reg_type,
        }
    }
}

/* PCIe Configuration Space Common Header */
pub const VENDOR_ID: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x00), CfgRegType::U16);
pub const DEVICE_ID: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x02), CfgRegType::U16);
pub const COMMAND: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x04), CfgRegType::U16);
pub const STATUS: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x06), CfgRegType::U16);
pub const REVISION_ID: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x08), CfgRegType::U8);
pub const PROG_IF: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x09), CfgRegType::U8);
pub const SUBCLASS: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x0a), CfgRegType::U8);
pub const CLASS: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x0b), CfgRegType::U8);
pub const CACHE_LINE_SIZE: CfgRegDesc =
    CfgRegDesc::new(PcieCfgOffset::new_const(0x0c), CfgRegType::U8);
pub const LATENCY_TIMER: CfgRegDesc =
    CfgRegDesc::new(PcieCfgOffset::new_const(0x0d), CfgRegType::U8);
pub const HEADER_TYPE: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x0e), CfgRegType::U8);
pub const BIST: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x0f), CfgRegType::U8);

pub mod endpoint {
    use super::*;

    pub const BAR0: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x10), CfgRegType::U32);
    pub const BAR1: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x14), CfgRegType::U32);
    pub const BAR2: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x18), CfgRegType::U32);
    pub const BAR3: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x1c), CfgRegType::U32);
    pub const BAR4: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x20), CfgRegType::U32);
    pub const BAR5: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x24), CfgRegType::U32);
    pub const SUBSYSTEM_VENDOR_ID: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x2c), CfgRegType::U16);
    pub const SUBSYSTEM_ID: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x2e), CfgRegType::U16);
    pub const EXPANSION_ROM_BASE_ADDRESS: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x30), CfgRegType::U32);
    pub const CAPABILITIES_POINTER: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x34), CfgRegType::U8);
    pub const INTERRUPT_LINE: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x3c), CfgRegType::U8);
    pub const INTERRUPT_PIN: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x3d), CfgRegType::U8);
    pub const MIN_GRANT: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x3e), CfgRegType::U8);
    pub const MAX_LATENCY: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x3f), CfgRegType::U8);
}

pub mod bridge {
    use super::*;

    pub const BAR0: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x10), CfgRegType::U32);
    pub const BAR1: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x14), CfgRegType::U32);
    pub const PRIMARY_BUS_NUMBER: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x18), CfgRegType::U8);
    pub const SECONDARY_BUS_NUMBER: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x19), CfgRegType::U8);
    pub const SUBORDINATE_BUS_NUMBER: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x1a), CfgRegType::U8);
    pub const IO_BASE: CfgRegDesc = CfgRegDesc::new(PcieCfgOffset::new_const(0x1c), CfgRegType::U8);
    pub const IO_LIMIT: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x1d), CfgRegType::U8);
    pub const SECONDARY_STATUS: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x1e), CfgRegType::U16);
    pub const MEMORY_BASE: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x20), CfgRegType::U16);
    pub const MEMORY_LIMIT: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x22), CfgRegType::U16);
    pub const PREFETCHABLE_MEMORY_BASE: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x24), CfgRegType::U16);
    pub const PREFETCHABLE_MEMORY_LIMIT: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x26), CfgRegType::U16);
    pub const PREFETCHABLE_MEMORY_BASE_UPPER: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x28), CfgRegType::U32);
    pub const PREFETCHABLE_MEMORY_LIMIT_UPPER: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x2c), CfgRegType::U32);
    pub const IO_BASE_UPPER: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x30), CfgRegType::U16);
    pub const IO_LIMIT_UPPER: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x32), CfgRegType::U16);
    pub const CAPABILITIES_POINTER: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x34), CfgRegType::U8);
    pub const EXPANSION_ROM_BASE_ADDRESS: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x38), CfgRegType::U32);
    pub const INTERRUPT_LINE: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x3c), CfgRegType::U8);
    pub const INTERRUPT_PIN: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x3d), CfgRegType::U8);
    pub const BRIDGE_CONTROL: CfgRegDesc =
        CfgRegDesc::new(PcieCfgOffset::new_const(0x3e), CfgRegType::U16);
}
