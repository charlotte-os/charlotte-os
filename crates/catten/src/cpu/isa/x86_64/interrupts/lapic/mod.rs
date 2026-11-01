use crate::cpu::isa::constants::msrs;
use crate::cpu::isa::interface::interrupts::LocalIntCtlrIfce;
use crate::cpu::isa::lp::{IntSrcDscr, LpId};

pub mod x2apic;
pub mod xapic;

pub enum Lapic {
    Xapic(xapic::Xapic),
    X2Apic(x2apic::X2Apic),
}

impl Lapic {
    pub fn is_x2apic_enabled() -> bool {
        const X2APIC_ENABLE_BIT: u64 = 1 << 10;
        unsafe { msrs::read(msrs::APIC_BASE_ADDRESS_REG) & X2APIC_ENABLE_BIT != 0 }
    }

    pub fn record_id() {
        if Self::is_x2apic_enabled() {
            x2apic::X2Apic::record_id();
        } else {
            xapic::Xapic::record_id();
        }
    }

    pub fn get_lic_id() -> u32 {
        if Self::is_x2apic_enabled() {
            x2apic::id::LapicId::get_local().physical
        } else {
            xapic::id::LapicId::get_local().physical as u32
        }
    }

    pub fn physical_id(lp_id: LpId) -> Option<u32> {
        if Self::is_x2apic_enabled() {
            x2apic::X2Apic::physical_id(lp_id)
        } else {
            Some(xapic::Xapic::physical_id(lp_id)? as u32)
        }
    }
}

impl LocalIntCtlrIfce for Lapic {
    type Error = super::Error;

    fn init_lp() {
        if Self::is_x2apic_enabled() {
            x2apic::X2Apic::init_lp();
        } else {
            xapic::Xapic::init_lp();
        }
    }

    fn send_unicast_ipi(target_lp: LpId, target_vector: IntSrcDscr) -> Result<(), Self::Error> {
        if Self::is_x2apic_enabled() {
            x2apic::X2Apic::send_unicast_ipi(target_lp, target_vector)
        } else {
            xapic::Xapic::send_unicast_ipi(target_lp, target_vector)
        }
    }

    fn signal_eoi() {
        if Self::is_x2apic_enabled() {
            x2apic::X2Apic::signal_eoi();
        } else {
            xapic::Xapic::signal_eoi();
        }
    }
}
