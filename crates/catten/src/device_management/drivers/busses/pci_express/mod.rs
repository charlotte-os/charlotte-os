pub mod capabilities;
pub mod device_class;
mod ecam;
pub mod topology;

#[derive(Debug)]
pub enum Error {
    InvalidLocation,
    PciCapabilitiesNotSupported,
    NotPciExpress,
    PciCapabilityNotFound,
    ValueOutOfRange,
    InvalidSegmentGroup,
    CfgOffsetOutOfRange,
    InvalidCapability,
}
