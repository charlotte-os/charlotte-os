//! # Universal Asynchronous Receiver/Transmitter (UART) Drivers

// pub mod ns16x50;

use core::fmt::Write;

pub use crate::device_management::drivers;
use crate::klib::io::Read;

pub type BaudRate = u32;

pub enum Error {
    GenericDriverError(drivers::Error),
    BaudNotConfigurable,
}

pub trait Uart: Read + Write {
    fn get_valid_baud_rates(&self) -> Result<&[BaudRate], Error>;
    fn set_baud_rate(&mut self, baud_rate: BaudRate) -> Result<BaudRate, Error>;
    fn get_baud_rate(&self) -> Result<BaudRate, Error>;
    fn enable_interrupts(&mut self) -> Result<(), Error>;
    fn disable_interrupts(&mut self) -> Result<(), Error>;
    fn are_interrupts_enabled(&self) -> Result<bool, Error>;
    fn is_interrupt_pending(&self) -> Result<bool, Error>;
}
