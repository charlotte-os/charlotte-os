//! # RISC-V Interrupt Management
//!
//! Local interrupts arrive through `stvec` and are classified by `scause`; external ones are
//! routed by a PLIC or APLIC. Neither controller is implemented yet.

pub mod dynamic;

use crate::cpu::isa::interface::interrupts::LocalIntCtlrIfce;
use crate::cpu::isa::lp::{IntSrcDscr, LpId};

#[derive(Debug)]
pub enum Error {
    InvalidLpId,
    IntVecUnassigned(u8),
    ArgIsFixedIntVec(u8),
}

/// The per-hart interrupt controller.
///
/// On RISC-V this is the supervisor trap CSRs plus whichever of SSIP, the ACLINT or the IMSIC the
/// platform provides for inter-hart interrupts.
pub struct SupervisorIntCtlr;

pub type LocalIntCtlr = SupervisorIntCtlr;

impl LocalIntCtlrIfce for SupervisorIntCtlr {
    type Error = Error;

    fn init_lp() {
        todo!("Point stvec at the trap entry and unmask the supervisor interrupt sources in sie.")
    }

    fn send_unicast_ipi(_target_lp: LpId, _target_vector: IntSrcDscr) -> Result<(), Self::Error> {
        todo!("Send an IPI via the SBI IPI extension, or the ACLINT/IMSIC where one is present.")
    }

    extern "C" fn signal_eoi() {
        todo!("Complete the interrupt at the PLIC/APLIC and clear the pending bit in sip.")
    }
}
