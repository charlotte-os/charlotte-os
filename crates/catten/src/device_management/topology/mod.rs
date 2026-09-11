use spin::LazyLock;

use crate::device_management::drivers::busses::pci_express::topology::{
    PcieLocation,
    PcieTopology,
};
use crate::environment::get_pcie_segment_groups;

pub static DEVICE_TOPOLOGY: LazyLock<DeviceTopology> = LazyLock::new(DeviceTopology::new);

/// A device function's location in the system's bus topology.
/// Used by drivers to access the device and configure access to it through its parent bus.
pub enum DeviceLocation {
    Pcie(PcieLocation),
    //Usb(usb::UsbAddress),
}

/// This struct represents the entire topology of peripheral devices in the system as seen by the
/// kernel device manager. It is used to query for devices and their locations in the system and to
/// provide information about the system's hardware configuration to userspace and kernel
/// subsystems.
#[derive(Debug)]
pub struct DeviceTopology {
    //fixed: Option<fixed_io::IoMap>,
    pub pcie: PcieTopology,
    //usb: Option<usb::UsbTopology>,
}

impl DeviceTopology {
    pub fn new() -> Self {
        DeviceTopology {
            pcie: PcieTopology::new(get_pcie_segment_groups()),
        }
    }
}

impl core::fmt::Display for DeviceTopology {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PCIe:\r\n")?;
        write!(f, "{}", self.pcie)
    }
}
