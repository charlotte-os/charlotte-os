//! # Device Drivers

use alloc::boxed::Box;
use core::fmt::Debug;

use crate::power_management::PowerState;

pub mod busses;
pub mod endpoints;
pub mod platform_devices;

pub enum Error {
    DeviceNotRecognized,
    InitializationFailed,
    DeinitializationFailed,
    DeviceAlreadyBoundToDriver,
}

/// The top level trait that all device class specific control planes must implement.
pub trait DeviceClassControlPlane: Debug {
    type Status: Debug;

    fn get_status(&self) -> Box<Self::Status>;
    fn transition_power_state(&mut self, state: PowerState) -> Result<(), Error>;
}
