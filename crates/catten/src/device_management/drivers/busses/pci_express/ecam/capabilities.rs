use alloc::vec::Vec;

use crate::device_management::drivers::busses::pci_express::ecam::headers::PcieCfgOffset;
use crate::device_management::drivers::busses::pci_express::topology::PcieFunction;

pub struct CapabilityIter<'a> {
    func_handle: &'a mut PcieFunction,
    offset: PcieCfgOffset,
    seen_offsets: Vec<PcieCfgOffset>,
}

impl<'a> CapabilityIter<'a> {
    const NEXT_OFFSET: u8 = 1;

    pub fn new(func_handle: &'a mut PcieFunction) -> Self {
        let offset = func_handle.get_capability_offset();
        Self {
            func_handle,
            offset,
            seen_offsets: Vec::new(),
        }
    }
}

impl<'a> Iterator for CapabilityIter<'a> {
    type Item = PcieCfgOffset;

    fn next(&mut self) -> Option<Self::Item> {
        let current_offset = self.offset;
        self.seen_offsets.push(current_offset);
        self.offset = self.func_handle.cfg_read8(self.offset + Self::NEXT_OFFSET).into();
        Some(self.offset)
    }
}
