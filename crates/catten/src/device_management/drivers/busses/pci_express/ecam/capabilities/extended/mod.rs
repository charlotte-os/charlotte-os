pub mod aer;
pub mod rsbar;

use super::*;
use crate::device_management::drivers::busses::pci_express::ecam::capabilities::standard::{
    PciCapabilityId,
    find_capability,
};
use crate::klib::bitwise::mask_shift_read;

/// This is the offset where the extended capability range starts in the PCI Express extended
/// configuration space.
const PCIE_EXT_CAP_RANGE_BASE: u16 = 256;

/// PCI Express Extended Capability IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PcieExtCapId {
    Null = 0x0000,
    AdvancedErrorReporting = 0x0001,
    AccessControlServices = 0x000d,
    // SR-IOV
    SingleRootIoVirtualization = 0x0010,
    ResizeableBaseAddressRegisters = 0x0011,
}

/// Represents the version and offset of a PCI Express Extended Capability.
/// The lower 4 bits represent the version, and the upper 12 bits represent the offset from the
/// beginning of the PCI Express extended configuration space in the ECAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PcieExtCapVerOffset(u16);

impl PcieExtCapVerOffset {
    pub fn new(offset: u16) -> Self {
        Self(offset)
    }

    pub fn raw(&self) -> u16 {
        self.0
    }

    pub fn get_version(&self) -> u8 {
        (self.0 & 0b1111) as u8
    }

    pub fn get_offset(&self) -> u16 {
        const OFFSET_MASK: u16 = 0xfff0;
        const OFFSET_SHIFT: u8 = 4;
        mask_shift_read(self.0, OFFSET_MASK, OFFSET_SHIFT)
    }
}

#[repr(C, packed)]
pub struct PcieExtCapabilityHeader {
    pub id: PcieExtCapId,
    pub next_ver: PcieExtCapVerOffset,
}

/// An iterator over the PCIe extended capabilities of a device.
/// It traverses the linked list of extended capabilities starting from the first capability offset
/// (PCIE_EXT_CAP_RANGE_BASE). Keeps track of seen offsets to avoid infinite loops in case of
/// malformed lists.
struct PcieExtCapIter {
    cfg_space: *const PcieCfgSpace,
    current_offset: PcieExtCapVerOffset,
    seen_offsets: Vec<PcieExtCapVerOffset>,
}

impl PcieExtCapIter {
    fn try_new(cfg_space: *const PcieCfgSpace) -> Result<Self, Error> {
        if let Ok(_) = find_capability(cfg_space, PciCapabilityId::PciExpress) {
            let starting_offset = PcieExtCapVerOffset::new(PCIE_EXT_CAP_RANGE_BASE);
            Ok(Self {
                cfg_space,
                current_offset: starting_offset,
                seen_offsets: Vec::new(),
            })
        } else {
            Err(Error::NotPciExpress)
        }
    }

    fn current(&mut self) -> *mut PcieExtCapabilityHeader {
        unsafe {
            // Safety: We got the address from the previous capability or the starting
            // offset, both of which are valid within the PCIe extended configuration space.
            (&raw mut self.cfg_space)
                .add(self.current_offset.get_offset() as usize)
                .cast::<PcieExtCapabilityHeader>()
        }
    }

    fn at_end(&self) -> bool {
        self.current_offset.get_offset() == 0
            || self.current_offset.get_offset() >= size_of::<PcieCfgSpace>() as u16
            || self.seen_offsets.contains(&self.current_offset)
    }
}

impl Iterator for PcieExtCapIter {
    type Item = *mut PcieExtCapabilityHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_end() {
            None
        } else {
            self.seen_offsets.push(self.current_offset);
            let current = self.current();
            self.current_offset = unsafe { (*current).next_ver };
            Some(current)
        }
    }
}

pub fn find_extended_capabilities(
    cfg_space: *const PcieCfgSpace,
    req_id: PcieExtCapId,
) -> Vec<*mut PcieExtCapabilityHeader> {
    if let Ok(mut iter) = PcieExtCapIter::try_new(cfg_space) {
        let mut matches = Vec::new();
        while let Some(cap) = iter.next() {
            unsafe {
                let ext_cap_id = cap.read_unaligned().id;
                if ext_cap_id == req_id {
                    matches.push(cap);
                }
            }
        }
        matches
    } else {
        Vec::new()
    }
}
