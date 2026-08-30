pub mod ethernet;
pub mod input_ctlr;
pub mod persistent_storage;
pub mod uart;

use crate::device_management::drivers::DeviceInterface;
use crate::device_management::topology::DeviceLocation;

pub trait EndpointControlPlane: DeviceInterface {
    fn get_location(&self) -> &DeviceLocation;
}
