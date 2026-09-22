pub mod i2c;
pub mod pci_express;
pub mod smbus;
pub mod spi;
pub mod usb;

/// Marker trait for bus control planes.
pub trait BusControlPlane {}
