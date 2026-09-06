use alloc::collections::btree_map::BTreeMap;

use crate::cpu::isa::constants::msrs;
use crate::cpu::isa::lp::LpId;
use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::klib::bitwise::{mask_from_len, mask_shift_read};

pub(super) static X2APIC_ID_TABLE: Mutex<BTreeMap<LpId, LapicId>> = Mutex::new(BTreeMap::new());

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct LapicId {
    pub physical: PhysicalLapicId,
    pub logical: LogicalLapicId,
}

impl LapicId {
    pub fn get_local() -> Self {
        let physical: PhysicalLapicId;
        let logical: u32;
        unsafe {
            core::arch::asm!(
                "rdmsr",
                in("ecx") msrs::x2apic::ID_REG,
                lateout("eax") physical,
                lateout("edx") _,
                options(nomem, nostack, preserves_flags),
            );
            core::arch::asm!(
                "rdmsr",
                in("ecx") msrs::x2apic::LOGICAL_DEST_REG,
                lateout("eax") logical,
                lateout("edx") _,
                options(nomem, nostack, preserves_flags),
            );
        }
        LapicId {
            physical,
            logical: LogicalLapicId {
                cluster_id: mask_shift_read(logical, mask_from_len(16), 16) as u16,
                apic_bitmask: mask_shift_read(logical, mask_from_len(16), 0) as u16,
            },
        }
    }
}
pub(super) type PhysicalLapicId = u32;
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub(super) struct LogicalLapicId {
    cluster_id: u16,
    apic_bitmask: u16,
}
