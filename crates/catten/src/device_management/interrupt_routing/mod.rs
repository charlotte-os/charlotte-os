//! # The Interrupt Router
//!
//! The interrupt router is responsible for the following:
//! - Allocating and deallocating interrupt vectors to devices
//! - Routing interrupts from devices to the appropriate interrupt vector
//! - Managing interrupt redirection tables for both platform level interrupt controllers and
//!   IOMMUs.
//! - Providing a unified interface for devices to register and unregister interrupt handlers.
//! - Ensuring that the interrupt service load is roughly balanced across all logical processors in
//!   the system.

use hashbrown::HashMap;

use crate::cpu::isa::interface::interrupts::DynIhMapIfce;
use crate::cpu::isa::interrupts::dynamic::DYN_IH_MAP;
use crate::cpu::isa::lp::{IntSrcDscr, LpId, WiredIntCtlrId, WiredIntCtlrSrcNum};
use crate::cpu::isa::{self};
use crate::device_management::drivers::busses::pci_express::topology::PcieLocation;
use crate::device_management::drivers::busses::pci_express::{self};

pub type InterruptHandler = extern "C" fn();

pub enum Error {
    InterruptVectorsExhausted,
    InterruptRedirectionEntriesExhausted,
    PcieError(pci_express::Error),
    IsaInterruptsError(isa::interrupts::Error),
}

impl From<isa::interrupts::Error> for Error {
    fn from(err: isa::interrupts::Error) -> Self {
        Error::IsaInterruptsError(err)
    }
}

/// External Interrupt Controller input source
#[derive(Clone, Copy, Debug)]
pub struct WiredSource {
    pub wired_ic_id: WiredIntCtlrId,
    pub source_num: WiredIntCtlrSrcNum,
    pub polarity: bool,
    pub latched: bool,
}

/// PCIe MSI source for an interrupt signal
#[derive(Clone, Copy, Debug)]
pub struct PcieMsiSource {
    pub location: PcieLocation,
    pub msi_num: u32,
}
/// PCIe MSI-X source for an interrupt signal
#[derive(Clone, Copy, Debug)]
pub struct PcieMsiXSource {
    pub location: PcieLocation,
    pub table_index: u32,
}
/// An enum representing the supported interrupt signal routing mechanisms
#[derive(Clone, Copy, Debug)]
pub enum InterruptSource {
    Wired(WiredSource),
    PcieMsi(PcieMsiSource),
    PcieMsiX(PcieMsiXSource),
}

impl InterruptSource {
    fn register_target(&self, target: InterruptTarget) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InterruptTarget {
    Processor {
        lp_id: LpId,
        discriminator: IntSrcDscr,
    },
    Remapper,
}

#[derive(Debug)]
pub struct InterruptRouter {
    routes: HashMap<InterruptSource, InterruptTarget>,
}

impl InterruptRouter {
    pub fn create_ext_int_route(
        &mut self,
        source: InterruptSource,
        handler: InterruptHandler,
    ) -> Result<InterruptTarget, Error> {
        /* Install the interrupt handler with a suitable target */
        let ih_map_lock = DYN_IH_MAP.write();
        let target = ih_map_lock.find_available_target().ok_or(Error::InterruptVectorsExhausted)?;
        todo!()
    }
}
