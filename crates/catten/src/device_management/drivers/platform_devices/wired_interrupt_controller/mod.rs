#[cfg(target_arch = "x86_64")]
pub mod ioapic;

use crate::cpu::isa::lp::WiredIntCtlrId;
use crate::device_management::interrupt_routing;
pub enum Error {
    InvalidSource,
    InvalidTarget,
}

pub trait WiredIntCtlr {
    fn set_source_target(
        &mut self,
        source: interrupt_routing::WiredSource,
        target: interrupt_routing::InterruptTarget,
    ) -> Result<(), Error>;
    fn clear_source_target(&mut self, source: interrupt_routing::WiredSource) -> Result<(), Error>;
    fn id(&self) -> WiredIntCtlrId;
    fn gsi_base(&self) -> u32;
}
