pub type PcieCfgOffset = u16;
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
const VENDOR_ID: CfgRegDesc = CfgRegDesc::new(0x00, CfgRegType::U16);
const DEVICE_ID: CfgRegDesc = CfgRegDesc::new(0x02, CfgRegType::U16);
const COMMAND: CfgRegDesc = CfgRegDesc::new(0x04, CfgRegType::U16);
const STATUS: CfgRegDesc = CfgRegDesc::new(0x06, CfgRegType::U16);
const REVISION_ID: CfgRegDesc = CfgRegDesc::new(0x08, CfgRegType::U8);
const PROG_IF: CfgRegDesc = CfgRegDesc::new(0x09, CfgRegType::U8);
const SUBCLASS: CfgRegDesc = CfgRegDesc::new(0x0a, CfgRegType::U8);
const CLASS: CfgRegDesc = CfgRegDesc::new(0x0b, CfgRegType::U8);
const CACHE_LINE_SIZE: CfgRegDesc = CfgRegDesc::new(0x0c, CfgRegType::U8);
const LATENCY_TIMER: CfgRegDesc = CfgRegDesc::new(0x0d, CfgRegType::U8);
const HEADER_TYPE: CfgRegDesc = CfgRegDesc::new(0x0e, CfgRegType::U8);
const BIST: CfgRegDesc = CfgRegDesc::new(0x0f, CfgRegType::U8);

pub mod endpoint {
    use super::{CfgRegDesc, CfgRegType};

    const BAR0: CfgRegDesc = CfgRegDesc::new(0x10, CfgRegType::U32);
    const BAR1: CfgRegDesc = CfgRegDesc::new(0x14, CfgRegType::U32);
    const BAR2: CfgRegDesc = CfgRegDesc::new(0x18, CfgRegType::U32);
    const BAR3: CfgRegDesc = CfgRegDesc::new(0x1c, CfgRegType::U32);
    const BAR4: CfgRegDesc = CfgRegDesc::new(0x20, CfgRegType::U32);
    const BAR5: CfgRegDesc = CfgRegDesc::new(0x24, CfgRegType::U32);
    const SUBSYSTEM_VENDOR_ID: CfgRegDesc = CfgRegDesc::new(0x2c, CfgRegType::U16);
    const SUBSYSTEM_ID: CfgRegDesc = CfgRegDesc::new(0x2e, CfgRegType::U16);
    const EXPANSION_ROM_BASE_ADDRESS: CfgRegDesc = CfgRegDesc::new(0x30, CfgRegType::U32);
    const CAPABILITIES_POINTER: CfgRegDesc = CfgRegDesc::new(0x34, CfgRegType::U8);
    const INTERRUPT_LINE: CfgRegDesc = CfgRegDesc::new(0x3c, CfgRegType::U8);
    const INTERRUPT_PIN: CfgRegDesc = CfgRegDesc::new(0x3d, CfgRegType::U8);
    const MIN_GRANT: CfgRegDesc = CfgRegDesc::new(0x3e, CfgRegType::U8);
    const MAX_LATENCY: CfgRegDesc = CfgRegDesc::new(0x3f, CfgRegType::U8);
}

pub mod bridge {
    use super::{CfgRegDesc, CfgRegType};

    const BAR0: CfgRegDesc = CfgRegDesc::new(0x10, CfgRegType::U32);
    const BAR1: CfgRegDesc = CfgRegDesc::new(0x14, CfgRegType::U32);
    const PRIMARY_BUS_NUMBER: CfgRegDesc = CfgRegDesc::new(0x18, CfgRegType::U8);
    const SECONDARY_BUS_NUMBER: CfgRegDesc = CfgRegDesc::new(0x19, CfgRegType::U8);
    const SUBORDINATE_BUS_NUMBER: CfgRegDesc = CfgRegDesc::new(0x1a, CfgRegType::U8);
    const IO_BASE: CfgRegDesc = CfgRegDesc::new(0x1c, CfgRegType::U8);
    const IO_LIMIT: CfgRegDesc = CfgRegDesc::new(0x1d, CfgRegType::U8);
    const SECONDARY_STATUS: CfgRegDesc = CfgRegDesc::new(0x1e, CfgRegType::U16);
    const MEMORY_BASE: CfgRegDesc = CfgRegDesc::new(0x20, CfgRegType::U16);
    const MEMORY_LIMIT: CfgRegDesc = CfgRegDesc::new(0x22, CfgRegType::U16);
    const PREFETCHABLE_MEMORY_BASE: CfgRegDesc = CfgRegDesc::new(0x24, CfgRegType::U16);
    const PREFETCHABLE_MEMORY_LIMIT: CfgRegDesc = CfgRegDesc::new(0x26, CfgRegType::U16);
    const PREFETCHABLE_MEMORY_BASE_UPPER: CfgRegDesc = CfgRegDesc::new(0x28, CfgRegType::U32);
    const PREFETCHABLE_MEMORY_LIMIT_UPPER: CfgRegDesc = CfgRegDesc::new(0x2c, CfgRegType::U32);
    const IO_BASE_UPPER: CfgRegDesc = CfgRegDesc::new(0x30, CfgRegType::U16);
    const IO_LIMIT_UPPER: CfgRegDesc = CfgRegDesc::new(0x32, CfgRegType::U16);
    const CAPABILITIES_POINTER: CfgRegDesc = CfgRegDesc::new(0x34, CfgRegType::U8);
    const EXPANSION_ROM_BASE_ADDRESS: CfgRegDesc = CfgRegDesc::new(0x38, CfgRegType::U32);
    const INTERRUPT_LINE: CfgRegDesc = CfgRegDesc::new(0x3c, CfgRegType::U8);
    const INTERRUPT_PIN: CfgRegDesc = CfgRegDesc::new(0x3d, CfgRegType::U8);
    const BRIDGE_CONTROL: CfgRegDesc = CfgRegDesc::new(0x3e, CfgRegType::U16);
}
