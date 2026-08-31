#[cfg(target_arch = "x86_64")]
pub mod ioapic;

use crate::cpu::isa::lp::WiredIntCtlrId;
use crate::device_management::interrupt_routing;
pub enum Error {
    InvalidSource,
    InvalidTarget,
    IdOutOfRange
}

pub trait WiredIntCtlr {
    fn num_sources(&self) -> u64;
    fn set_target(
        &mut self,
        source: interrupt_routing::WiredSource,
        target: interrupt_routing::InterruptTarget,
    ) -> Result<(), Error>;
    fn clear_target(&mut self, source: interrupt_routing::WiredSource) -> Result<(), Error>;
    fn id(&self) -> WiredIntCtlrId;
}
