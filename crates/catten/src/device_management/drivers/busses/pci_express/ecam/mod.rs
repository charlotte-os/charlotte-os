//! # PCI Express Enhanced Configuration Access Mechanism (ECAM)
pub(super) mod capabilities;
pub(super) mod headers;

use hashbrown::HashMap;
use spin::LazyLock;

use crate::cpu::isa::lp::ops::{
    config_read8,
    config_read16,
    config_read32,
    config_write8,
    config_write16,
    config_write32,
};
use crate::device_management::drivers::busses::pci_express::topology::{
    PcieLocation,
    PcieSegmentGroupNum,
};
