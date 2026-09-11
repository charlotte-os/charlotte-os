//! # AArch64 Generic Interrupt Controller
//!
//! Catten targets GICv3 and later, where each logical processor has a redistributor and the CPU
//! interface is reached through system registers (`ICC_*_EL1`) rather than MMIO.

use crate::cpu::isa::interface::interrupts::LocalIntCtlrIfce;
use crate::cpu::isa::interrupts::Error;
use crate::cpu::isa::lp::{IntSrcDscr, LpId};

pub type LocalIntCtlr = GicRedist;

/// The GIC redistributor and CPU interface belonging to one logical processor.
pub struct GicRedist;

impl LocalIntCtlrIfce for GicRedist {
    type Error = Error;

    fn init_lp() {
        todo!(
            "Wake the redistributor by clearing GICR_WAKER.ProcessorSleep, then enable the system \
             register CPU interface through ICC_SRE_EL1 and ICC_IGRPEN1_EL1."
        )
    }

    fn send_unicast_ipi(_target_lp: LpId, _target_vector: IntSrcDscr) -> Result<(), Self::Error> {
        todo!("Write the target's affinity and the SGI INTID to ICC_SGI1R_EL1.")
    }

    extern "C" fn signal_eoi() {
        todo!("Write the acknowledged INTID to ICC_EOIR1_EL1, and ICC_DIR_EL1 when EOImode is 1.")
    }
}
