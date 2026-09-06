use super::*;
/// PCI local bus capability IDs which are also used with PCI Express.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PciCapabilityId {
    Null = 0x00,
    PowerManagement = 0x01,
    MessageSignaledInterrupts = 0x05,
    /// Subsystem Vendor ID and Subsystem Device ID
    SubsystemVidDid = 0x0d,
    PciExpress = 0x10,
    // MSI-X
    MessageSignaledInterruptsExtended = 0x11,
}

pub type PciCapabilityOffset = u8;

/// An overlay struct for the PCI capability header within the configuration space of a PCI
/// device. It provides the ID of the capability and the offset to the next capability in
/// the linked list.
///
/// This structure is not created but used to access MMIO via pointer. That pointer should be
/// casted to the appropriate capability type once the header `id` field has been read.
#[repr(C, packed)]
pub struct PciCapabilityHeader {
    pub id: PciCapabilityId,
    pub next: PciCapabilityOffset,
}

/// An iterator over the PCI capabilities of a device.
/// It traverses the linked list of capabilities starting from the first capability offset.
/// Keeps track of seen offsets to avoid infinite loops in case of malformed lists.
struct PciCapabilityIter {
    cfg_space: *const PcieCfgSpace,
    current_offset: PciCapabilityOffset,
    seen_offsets: Vec<PciCapabilityOffset>,
}

impl PciCapabilityIter {
    fn try_new(cfg_space: *const PcieCfgSpace) -> Result<Self, Error> {
        // Bail early if capabilities are not supported
        if core::hint::unlikely(!unsafe { (*cfg_space).header.common.are_capabilities_supported() })
        {
            return Err(Error::PciCapabilitiesNotSupported);
        }

        let mut starting_offset = 0;
        if core::hint::unlikely(unsafe { (*cfg_space).header.common.is_bridge() }) {
            if let Some(offset) = unsafe { (*cfg_space).header.bridge.get_capabilities_offset() } {
                starting_offset = offset;
            }
        } else {
            if let Some(offset) = unsafe { (*cfg_space).header.endpoint.get_capabilities_offset() }
            {
                starting_offset = offset;
            }
        }
        Ok(Self {
            cfg_space,
            current_offset: starting_offset,
            seen_offsets: Vec::new(),
        })
    }

    fn current(&self) -> *const PciCapabilityHeader {
        unsafe {
            // Safety: We got the address from the previous capability or the starting
            // offset, both of which are valid within the PCI configuration space.
            (&raw const self.cfg_space)
                .add(self.current_offset as usize)
                .cast::<PciCapabilityHeader>()
        }
    }
}

impl Iterator for PciCapabilityIter {
    type Item = &'static PciCapabilityHeader;

    fn next(&mut self) -> Option<&'static PciCapabilityHeader> {
        let curr_hdr = unsafe { self.cfg_space.byte_add(self.current_offset as usize) }
            as *const PciCapabilityHeader;
        if curr_hdr.is_null() || self.seen_offsets.contains(&self.current_offset) {
            return None;
        } else {
            self.seen_offsets.push(self.current_offset);
            self.current_offset = unsafe { (*curr_hdr).next };
            Some(unsafe { &*curr_hdr })
        }
    }
}

pub fn find_capability(
    cfg_space: *const PcieCfgSpace,
    id: PciCapabilityId,
) -> Result<*const PciCapabilityHeader, Error> {
    let iter = PciCapabilityIter::try_new(cfg_space)?;
    for cap in iter {
        if cap.id == id {
            return Ok(cap);
        }
    }
    Err(Error::PciCapabilityNotFound)
}
