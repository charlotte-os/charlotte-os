use crate::cpu::isa::interrupts::Error;
use crate::cpu::isa::lp::{IntSrcDscr, LpId};
use crate::device_management::interrupt_routing::InterruptHandler;

/// Dynamic Interrupt Dispatcher Interface
pub trait DynIhMapIfce {
    /// Set the interrupt handler for a given logical processor and vector
    fn set_dyn_ih(
        &self,
        lp: LpId,
        vector: IntSrcDscr,
        handler: InterruptHandler,
    ) -> Result<(), Error>;
    /// Get the interrupt handler for a given vector
    /// Note: must be #[unsafe(no_mangle)] and extern "C" to be callable from assembly code
    extern "C" fn get_local_dyn_ih(&self, vector: IntSrcDscr) -> Option<InterruptHandler>;
    /// Clear the assigned interrupt handler for a given logical processor and vector if there is
    /// one
    fn clear_dyn_ih(&self, lp: LpId, vector: IntSrcDscr) -> Result<(), Error>;
    /// Find an available dynamic vector
    fn find_available_vector(&self) -> Option<IntSrcDscr>;
}

/// Local Interrupt Controller Interface
pub trait LocalIntCtlrIfce {
    type Error;

    /// Initialize the local interrupt controller for the current logical processor
    fn init_lp();
    /// Send an inter-processor interrupt to the specified logical processor
    fn send_unicast_ipi(target_lp: LpId, target_vector: IntSrcDscr) -> Result<(), Self::Error>;
    /// Signal End of Interrupt
    extern "C" fn signal_eoi();
}

pub trait ExternalInterruptControllerIfce {
    type EicPinNum;
    type Error;

    /// Initialize the external interrupt controller
    fn init(&mut self);
    /// Wire-up an external interrupt to a logical processor and vector
    fn setup_ext_int(
        &mut self,
        lp: LpId,
        vector: IntSrcDscr,
        pin_num: Self::EicPinNum,
        active_low: bool,
        level_triggered: bool,
        mask_state: bool,
    ) -> Result<(), Self::Error>;
    /// Set the mask state of an external interrupt pin
    fn set_ext_int_mask_state(
        &mut self,
        pin_num: Self::EicPinNum,
        mask_state: bool,
    ) -> Result<(), Self::Error>;
}
