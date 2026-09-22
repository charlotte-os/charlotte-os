pub mod ethernet;
pub mod input_ctlr;
pub mod persistent_storage;
pub mod uart;

use crate::device_management::DeviceLocation;
use crate::device_management::drivers::DeviceClassControlPlane;

pub trait EndpointControlPlane: DeviceClassControlPlane {
    fn get_location(&self) -> &DeviceLocation;
}
