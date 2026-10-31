use alloc::collections::btree_map::BTreeMap;

use crate::cpu::isa::lp::LpId;
use crate::cpu::multiprocessor::spin::mutex::Mutex;

pub(super) static XAPIC_ID_TABLE: Mutex<BTreeMap<LpId, LapicId>> = Mutex::new(BTreeMap::new());

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct LapicId {
    pub physical: PhysicalLapicId,
    pub logical: LogicalLapicId,
}

impl LapicId {
    pub const XAPIC_ID_SHIFT: u32 = 24;

    pub fn get_local() -> Self {
        let physical: PhysicalLapicId;
        let logical: LogicalLapicId;
        unsafe {
            physical = (super::io::xapic_io_read32(super::xapic::ID_REG) >> Self::XAPIC_ID_SHIFT)
                as PhysicalLapicId;
            logical = (super::io::xapic_io_read32(super::xapic::LOGICAL_DEST_REG)
                >> Self::XAPIC_ID_SHIFT) as LogicalLapicId;
        }
        LapicId {
            physical,
            logical,
        }
    }
}
pub(super) type PhysicalLapicId = u8;
pub(super) type LogicalLapicId = u8;
