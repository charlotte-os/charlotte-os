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
}

#[allow(unused)]
const MAX_SEGMENT_GROUPS: usize = 1 << 16; // 65536 segment groups
const MAX_DEVICES_PER_BUS: usize = 32;
const MAX_FUNCTIONS_PER_DEVICE: usize = 8;
