use crate::device_management::drivers::busses::pci_express::ecam::headers;

/// The size of the configuration space for legacy PCI and PCI-X local bus devices as may be used
/// via PCIe to PCI/PCI-X bridges.
const PCI_LOCAL_CFG_SPACE_SIZE: usize = 256;
/// The size of the full PCI Express configuration space, including both the legacy configuration
/// space and extended capability space as defined by the PCI Express specification.
pub const PCIE_CFG_SPACE_SIZE: usize = 4096;

/// The extended configuration space of a PCIe device should only be accessed if the PCIe capability
/// is present in the normal capability space. If the PCIe capability is not present, then this
/// configuration space is reserved and should not be accessed. Therefore, we can represent this
/// configuration space as a union of the PCI local configuration space (which is used for non-PCIe
/// devices) and the PCIe extended configuration space (which is used for PCIe devices).
///
/// This representation correctly makes variant access unsafe, as the caller must check the presence
/// of the PCIe capability before accessing the extended configuration space and the compiler cannot
/// enforce this check. As such it must be handled correctly as an implementation detail of the PCIe
/// configuration space overlay struct while exposing a safe interface to the rest of the system.
pub union ExtCapSpace {
    pci_local: (),
    pcie: [u8; PCIE_CFG_SPACE_SIZE - PCI_LOCAL_CFG_SPACE_SIZE],
}

#[repr(C, packed)]
/// An overlay struct representing the entire 4KB configuration space of a PCIe device in an ECAM
pub struct PcieCfgSpace {
    pub header: headers::CfgHeader,
    pub capability_space:
        [u8; PCI_LOCAL_CFG_SPACE_SIZE - core::mem::size_of::<headers::CfgHeader>()],
    /* The extended capability space should only be used if the PCIe capability is present in
     * the normal capability space. Otherwise this configuration space is for a PCI local bus
     * device which does not support PCIe extended capabilities. */
    pub ext_capability_space: ExtCapSpace,
}

impl PcieCfgSpace {
    /// Determines if there is device present at the device slot corresponding to this configuration
    /// space.
    pub fn has_device_present(&self) -> bool {
        unsafe { self.header.common.is_device_present() }
    }

    /// Determines if the device corresponding to this configuration space is a PCI(e) to PCI(e)
    /// bridge.
    pub fn device_is_bridge(&self) -> bool {
        unsafe { self.header.common.is_bridge() }
    }

    /// Determines if the device corresponding to this configuration space is a multifunction device
    /// with multiple functions at the same device slot, by checking the multifunction bit in the
    /// header type field of the endpoint header for function 0 of the device slot. If there is no
    /// device present at function 0, then this function will accurately return false, as there
    /// cannot be multiple functions without a device present in the first place.
    pub fn device_is_multifunction(&self) -> bool {
        unsafe { self.header.common.is_multi_function() }
    }
}
