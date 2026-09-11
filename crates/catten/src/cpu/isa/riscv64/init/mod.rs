//! # RISC-V System Initialization

use crate::cpu::isa::interface::init::InitInterface;

pub struct IsaInitializer;

impl InitInterface for IsaInitializer {
    type Error = core::convert::Infallible;

    fn init_bsp() -> Result<(), Self::Error> {
        todo!("Set up the trap vector, the per-hart control block and the supervisor CSRs.")
    }

    fn init_ap() -> Result<(), Self::Error> {
        todo!("Bring a secondary hart up through the SBI HSM extension and initialise its CSRs.")
    }

    fn deinit() -> Result<(), Self::Error> {
        Ok(())
    }
}
