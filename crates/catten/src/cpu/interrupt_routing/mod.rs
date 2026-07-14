//! # The Interrupt Routing Manager
//!
//! The interrupt routing manager is responsible for the following:
//! - Allocating and deallocating interrupt vectors to devices
//! - Routing interrupts from devices to the appropriate interrupt vector
//! - Managing interrupt redirection tables for both platform level interrupt controllers and
//!   IOMMUs.
//! - Providing a unified interface for devices to register and unregister interrupt handlers.
//! - Ensuring that the interrupt service load is roughly balanced across all logical processors in
//!   the system.

use alloc::collections::btree_map::BTreeMap;

use hashbrown::HashMap;

use crate::{
    cpu::isa::{
        self,
        constants::interrupt_vectors::{
            DYN_VEC_START_OFFSET,
            DYN_VECS_PER_LP,
        },
        interface::interrupts::DynInterruptDispatcherIfce,
        interrupts::dynamic::DYN_IH_MATRIX,
        lp::{
            EicId,
            EicPinNum,
            InterruptSourceDiscriminator,
            LpId,
        },
    },
    device_management::drivers::busses::pci_express::{
        self,
        topology::PcieLocation,
    },
};

pub type InterruptHandler = extern "C" fn();

pub enum Error {
    InterruptVectorsExhausted,
    InterruptRedirectionTableFull,
    PcieError(pci_express::Error),
    IsaInterruptsError(isa::interrupts::Error),
}

impl From<isa::interrupts::Error> for Error {
    fn from(err: isa::interrupts::Error) -> Self {
        Error::IsaInterruptsError(err)
    }
}

/// External Interrupt Controller input source
#[derive(Default, Clone, Debug)]
pub struct EicSource {
    pub pic_id: EicId,
    pub pin_num: EicPinNum,
}
/// PCIe MSI source for an interrupt signal
#[derive(Default, Clone, Debug)]
pub struct PcieMsiSource {
    pub location: PcieLocation,
    pub msi_num: u32,
}
/// PCIe MSI-X source for an interrupt signal
#[derive(Default, Clone, Debug)]
pub struct PcieMsiXSource {
    pub location: PcieLocation,
    pub table_index: u32,
}
/// An enum representing the supported interrupt signal routing mechanisms
#[derive(Clone, Debug)]
pub enum InterruptRouter {
    ExternalInterruptController(EicSource),
    PcieMsi(PcieMsiSource),
    PcieMsiX(PcieMsiXSource),
}
#[derive(Default, Clone, Debug)]
pub struct InterruptSignalType {
    pub level_triggered: bool,
    pub active_level: bool,
}
#[derive(Clone, Debug)]
pub struct InterruptInput {
    pub router: InterruptRouter,
    pub signal_type: InterruptSignalType,
}

#[derive(Default, Clone, Debug)]
pub struct InterruptRoutingManager {
    routes: HashMap<LpId, BTreeMap<InterruptSourceDiscriminator, InterruptInput>>,
}

pub struct InterruptTarget {
    lp_id: LpId,
    discriminator: InterruptSourceDiscriminator,
}

impl InterruptRoutingManager {
    pub fn register_external_interrupt(
        &mut self,
        input: InterruptInput,
        handler: InterruptHandler,
    ) -> Result<InterruptTarget, Error> {
        /* Find the least loaded logical processor and a free dynamic interrupt discriminator
         * value on it */
        let target_lp = self.least_loaded_lp();
        let vector = self.find_free_vector(target_lp).ok_or(Error::InterruptVectorsExhausted)?;
        /* Set the dynamic interrupt handler for the chosen logical processor and vector and
         * record the mapping in the routing table */
        DYN_IH_MATRIX.set_dyn_ih(target_lp, vector, handler)?;
        self.routes.entry(target_lp).or_default().insert(vector, input);
        /* Map the route on the inbound interrupt signal routing mechanism */
        let target = InterruptTarget {
            lp_id: target_lp,
            discriminator: vector,
        };

        todo!("Implement IOMMU interrupt redirection.");

        /* TODO: Implement routing for all possible interrupt signal routing mechanisms to the
        created IOMMU redirection entry. */
        input.route(&target)?;

        Ok(target)
    }

    fn least_loaded_lp(&self) -> LpId {
        let mut lp: LpId = 0;

        for (lp_id, routes) in &self.routes {
            if routes.len() < self.routes[&lp].len() {
                lp = *lp_id;
            }
        }
        lp
    }

    fn find_free_vector(&self, lp: LpId) -> Option<InterruptSourceDiscriminator> {
        for v in DYN_VEC_START_OFFSET..DYN_VECS_PER_LP {
            let vector = InterruptSourceDiscriminator::from(v);
            if !self.routes[&lp].contains_key(&vector) {
                return Some(vector);
            }
        }
        None
    }
}
