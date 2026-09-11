//! # AArch64 Dynamic Interrupt Dispatch
//!
//! The matrix of per-LP handlers that external interrupts are dispatched through once the GIC
//! distributor can route them.

use spin::LazyLock;

use crate::cpu::isa::interface::interrupts::DynIhMapIfce;
use crate::cpu::isa::interrupts::Error;
use crate::cpu::isa::lp::{IntSrcDscr, LpId};
use crate::cpu::multiprocessor::spin::rwlock::RwLock;
use crate::device_management::interrupt_routing::{InterruptHandler, InterruptTarget};

/// The instance of the dynamic interrupt handler matrix.
#[unsafe(no_mangle)]
pub static DYN_IH_MAP: LazyLock<RwLock<DynIhMap>> = LazyLock::new(|| RwLock::new(DynIhMap::new()));

pub struct DynIhMap {
    _private: (),
}

impl DynIhMap {
    pub fn new() -> Self {
        DynIhMap {
            _private: (),
        }
    }
}

impl Default for DynIhMap {
    fn default() -> Self {
        Self::new()
    }
}

impl DynIhMapIfce for DynIhMap {
    fn set_dyn_ih(
        &mut self,
        _lp: LpId,
        _vector: IntSrcDscr,
        _handler: InterruptHandler,
    ) -> Result<(), Error> {
        todo!("Record the handler once AArch64 GIC INTIDs are allocated per LP.")
    }

    extern "C" fn get_local_dyn_ih(&self, _vector: IntSrcDscr) -> Option<InterruptHandler> {
        todo!("Look the handler up from the IRQ dispatcher on the current LP.")
    }

    fn clear_dyn_ih(&mut self, _lp: LpId, _vector: IntSrcDscr) -> Result<(), Error> {
        todo!("Release a previously assigned GIC INTID.")
    }

    fn find_available_target(&self) -> Option<InterruptTarget> {
        todo!("Pick an LP and a free INTID for a newly routed external interrupt.")
    }
}
