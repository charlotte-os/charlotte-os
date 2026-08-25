//! # The Device Manager
pub mod drivers;
pub mod hw_interface;
pub mod interrupt_routing;
pub mod topology;

use alloc::{
    sync::Arc,
    vec::Vec,
};

use drivers::{
    busses::*,
    endpoints::*,
};

pub struct DeviceControlPlaneTable {
    pub pcie_root_complex: Vec<pci_express::topology::PcieSegmentGroup>,
    pub uart: Vec<Arc<dyn uart::Uart>>,
}
