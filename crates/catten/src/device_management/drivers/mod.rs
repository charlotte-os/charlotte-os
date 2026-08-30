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

pub trait DeviceInterface {
    type Status: Debug;

    fn get_status(&self) -> Box<Self::Status>;
    fn transition_power_state(&mut self, state: PowerState) -> Result<(), Error>;
}
