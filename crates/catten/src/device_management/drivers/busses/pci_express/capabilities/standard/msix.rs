use crate::device_management::drivers::busses::pci_express::ecam::headers::PcieCfgOffset;
use crate::device_management::drivers::busses::pci_express::topology::PcieFunction;
use crate::device_management::drivers::busses::pci_express::{self};

const MSIX_CAP_ID: u8 = 0x11;

const MESSAGE_CONTROL_OFFSET: PcieCfgOffset = PcieCfgOffset::new_const(0x02);
const TABLE_OFFSET_REG_OFFSET: PcieCfgOffset = PcieCfgOffset::new_const(0x04);
const PBA_OFFSET_REG_OFFSET: PcieCfgOffset = PcieCfgOffset::new_const(0x08);

const MSG_CTL_MSIX_ENABLE_SHIFT: u32 = 15;
const MSG_CTL_MASK_FUNC_SHIFT: u32 = 14;
const OFFSET_REG_BIR_MASK: u32 = 0b111;
const OFFSET_REG_OFFSET_MASK: u32 = !OFFSET_REG_BIR_MASK;

pub struct MsiXCapHandle<'a> {
    func_handle: &'a mut PcieFunction,
    cap_offset: PcieCfgOffset,
}

impl<'a> MsiXCapHandle<'a> {
    pub fn try_new(
        func_handle: &'a mut PcieFunction,
        cap_offset: PcieCfgOffset,
    ) -> Result<Self, pci_express::Error> {
        let instance = Self {
            func_handle,
            cap_offset,
        };
        if instance.func_handle.cfg_read8(instance.cap_offset) == MSIX_CAP_ID {
            Ok(instance)
        } else {
            Err(pci_express::Error::InvalidCapability)
        }
    }

    pub fn get_table_size(&self) -> u16 {
        let message_control = self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        message_control & ((1u16 << 11u16) - 1u16)
    }

    pub fn is_msix_enabled(&self) -> bool {
        let message_control = self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        (message_control & (1 << MSG_CTL_MSIX_ENABLE_SHIFT)) != 0
    }

    pub fn enable_msix(&mut self) {
        let mut message_control =
            self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        message_control |= 1 << MSG_CTL_MSIX_ENABLE_SHIFT;
        self.func_handle.cfg_write16(self.cap_offset + MESSAGE_CONTROL_OFFSET, message_control);
    }

    pub fn disable_msix(&mut self) {
        let mut message_control =
            self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        message_control &= !(1 << MSG_CTL_MSIX_ENABLE_SHIFT);
        self.func_handle.cfg_write16(self.cap_offset + MESSAGE_CONTROL_OFFSET, message_control);
    }

    pub fn is_function_masked(&self) -> bool {
        let message_control = self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        (message_control & (1 << MSG_CTL_MASK_FUNC_SHIFT)) != 0
    }

    pub fn mask_function(&mut self) {
        let mut message_control =
            self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        message_control |= 1 << MSG_CTL_MASK_FUNC_SHIFT;
        self.func_handle.cfg_write16(self.cap_offset + MESSAGE_CONTROL_OFFSET, message_control);
    }

    pub fn unmask_function(&mut self) {
        let mut message_control =
            self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        message_control &= !(1 << MSG_CTL_MASK_FUNC_SHIFT);
        self.func_handle.cfg_write16(self.cap_offset + MESSAGE_CONTROL_OFFSET, message_control);
    }

    pub fn find_msix_table(&self) -> MsixTableLocation {
        let table_offset = self.func_handle.cfg_read32(self.cap_offset + TABLE_OFFSET_REG_OFFSET);
        let bar_index = (table_offset & OFFSET_REG_BIR_MASK) as u8;
        let offset = table_offset & OFFSET_REG_OFFSET_MASK;
        MsixTableLocation {
            bar_index,
            offset,
        }
    }

    pub fn find_pba_table(&self) -> MsixTableLocation {
        let pba_offset = self.func_handle.cfg_read32(self.cap_offset + PBA_OFFSET_REG_OFFSET);
        let bar_index = (pba_offset & OFFSET_REG_BIR_MASK) as u8;
        let offset = pba_offset & OFFSET_REG_OFFSET_MASK;
        MsixTableLocation {
            bar_index,
            offset,
        }
    }
}

pub struct MsixTableLocation {
    pub bar_index: u8,
    pub offset: u32,
}
