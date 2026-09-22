//! # The Device Manager
pub mod drivers;
pub mod hw_interface;
pub mod interrupt_routing;

#[cfg(feature = "acpi")]
use alloc::ffi::CString;
use alloc::sync::Arc;
use alloc::vec::Vec;

use drivers::busses::*;
use drivers::endpoints::*;

/// Contains the kernel abstracted Device Class Control Plane (DCCP) interfaces most of which are
/// trait objects. These DCCPs are the means by which both kernel subsystems outside the DM and
/// userspace applications interact with the underlying hardware. In the Catten Kernel driver model,
/// drivers are built to facilitate interaction with the underlying hardware through these DCCPs.
/// DCCPs should generalize exact device model specific hardware interfaces while still allowing
/// their users to have fine-grained control over the underlying hardware to the fullest extent
/// possible. The marker trait `DeviceClassControlPlane` should be used as a trait
/// bound on all device class specific DCCP traits and the few concrete (single implementation)
/// DCCPs.
pub struct DeviceControlPlaneTable {
    pub pcie_root_complex: Vec<pci_express::topology::PcieSegmentGroup>,
    pub uart: Vec<Arc<dyn uart::Uart>>,
}

pub enum DeviceLocation {
    #[cfg(feature = "acpi")]
    AcpiNamespace(CString),
    #[cfg(feature = "devicetree")]
    Devicetree(DevicetreePath),
    Pcie(pci_express::topology::PcieLocation),
    // Usb(usb::UsbAddress),
    // I2c(i2c::I2cAddress),
    // Spi(spi::SpiAddress),
}
