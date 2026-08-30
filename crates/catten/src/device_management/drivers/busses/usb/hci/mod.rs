use alloc::sync::Arc;

use crate::device_management::drivers::DeviceInterface;
use crate::device_management::drivers::busses::BusControlPlane;
use crate::device_management::drivers::endpoints::EndpointControlPlane;

pub mod xhci;

pub trait UsbHciControlPlane: BusControlPlane + EndpointControlPlane {
    fn get_usb_version(self: Arc<Self>) -> u8;
    fn get_maximum_speed(self: Arc<Self>) -> u8;
    fn get_number_of_ports(self: Arc<Self>) -> u8;
    fn get_port_status(self: Arc<Self>, port_number: u8) -> u8;
}
