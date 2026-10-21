use crate::device_management::drivers::busses::pci_express;
use crate::device_management::drivers::busses::pci_express::ecam::headers::PcieCfgOffset;
use crate::device_management::drivers::busses::pci_express::topology::PcieFunction;
use crate::memory::PhysicalAddress;

const MSI_CAP_ID: u8 = 0x05;

/// MSI capability offsets within the PCIe configuration space
const MESSAGE_CONTROL_OFFSET: u8 = 0x02;
const MSI_ADDR_OFFSET: u8 = 0x04;
const MSI_ADDR_UPPER_OFFSET: u8 = 0x08;

/// MSI message control register bit shifts and masks
const MSI_ENABLE_BIT_SHIFT: u16 = 0;
const MULTIPLE_MESSAGE_CAPABLE_MASK: u16 = 0b1110;
const MULTIPLE_MESSAGE_ENABLE_SHIFT: u16 = 1;
const ADDR_64BIT_CAPABLE_SHIFT: u16 = 7;
const PER_VECTOR_MASKING_CAPABLE_SHIFT: u16 = 8;
const EXTENDED_MESSAGE_DATA_CAPABLE_SHIFT: u16 = 9;

#[derive(Debug)]
pub enum Error {
    InvalidVecReqCount,
    InvalidAllocVecCount,
    ExtendedMessageDataNotSupported,
    AddressOutOfRange,
}

pub struct MsiCapHandle<'a> {
    func_handle: &'a mut PcieFunction,
    cap_offset: PcieCfgOffset,
}

impl<'a> MsiCapHandle<'a> {
    pub fn try_new(
        func_handle: &'a mut PcieFunction,
        cap_offset: PcieCfgOffset,
    ) -> Result<Self, crate::device_management::drivers::busses::pci_express::Error> {
        let instance = Self {
            func_handle,
            cap_offset,
        };
        if instance.func_handle.cfg_read8(instance.cap_offset) == MSI_CAP_ID {
            Ok(instance)
        } else {
            Err(crate::device_management::drivers::busses::pci_express::Error::InvalidCapability)
        }
    }

    pub fn is_msi_enabled(&self) -> bool {
        let message_control = self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        (message_control & (1 << MSI_ENABLE_BIT_SHIFT)) != 0
    }

    pub fn enable_msi(&mut self) {
        let message_control = self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        let new_message_control = message_control | (1 << MSI_ENABLE_BIT_SHIFT);
        self.func_handle.cfg_write16(self.cap_offset + MESSAGE_CONTROL_OFFSET, new_message_control);
    }

    pub fn disable_msi(&mut self) {
        let message_control = self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        let new_message_control = message_control & !(1 << MSI_ENABLE_BIT_SHIFT);
        self.func_handle.cfg_write16(self.cap_offset + MESSAGE_CONTROL_OFFSET, new_message_control);
    }

    pub fn get_vec_req_count(&self) -> Result<u8, Error> {
        let message_control = self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        let val =
            (message_control & MULTIPLE_MESSAGE_CAPABLE_MASK) >> MULTIPLE_MESSAGE_ENABLE_SHIFT;
        if val < 6 {
            Ok(1 << val)
        } else {
            Err(Error::InvalidVecReqCount)
        }
    }

    pub fn set_alloc_vec_count(&mut self, count: u8) -> Result<(), Error> {
        if count == 0 || count > 32 || count.count_ones() != 1 {
            Err(Error::InvalidAllocVecCount)
        } else {
            let encoded_val = (count >> 1) as u16;
            let message_control =
                self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
            let new_message_control = (message_control & !MULTIPLE_MESSAGE_CAPABLE_MASK)
                | (encoded_val << MULTIPLE_MESSAGE_ENABLE_SHIFT);
            self.func_handle
                .cfg_write16(self.cap_offset + MESSAGE_CONTROL_OFFSET, new_message_control);
            Ok(())
        }
    }

    pub fn is_64bit_addr_capable(&self) -> bool {
        let message_control = self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        (message_control & (1 << ADDR_64BIT_CAPABLE_SHIFT)) != 0
    }

    pub fn is_per_vector_masking_capable(&self) -> bool {
        let message_control = self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        (message_control & (1 << PER_VECTOR_MASKING_CAPABLE_SHIFT)) != 0
    }

    pub fn is_extended_message_data_capable(&self) -> bool {
        let message_control = self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        (message_control & (1 << EXTENDED_MESSAGE_DATA_CAPABLE_SHIFT)) != 0
    }

    pub fn is_extended_message_data_enabled(&self) -> bool {
        let message_control = self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
        (message_control & (1 << EXTENDED_MESSAGE_DATA_CAPABLE_SHIFT)) != 0
    }

    pub fn enable_extended_message_data(&mut self) -> Result<(), Error> {
        if !self.is_extended_message_data_capable() {
            Err(Error::ExtendedMessageDataNotSupported)
        } else {
            if !self.is_extended_message_data_enabled() {
                let message_control =
                    self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
                let new_message_control =
                    message_control | (1 << EXTENDED_MESSAGE_DATA_CAPABLE_SHIFT);
                self.func_handle
                    .cfg_write16(self.cap_offset + MESSAGE_CONTROL_OFFSET, new_message_control);
            }
            Ok(())
        }
    }

    pub fn disable_extended_message_data(&mut self) -> Result<(), Error> {
        if !self.is_extended_message_data_capable() {
            Err(Error::ExtendedMessageDataNotSupported)
        } else {
            if self.is_extended_message_data_enabled() {
                let message_control =
                    self.func_handle.cfg_read16(self.cap_offset + MESSAGE_CONTROL_OFFSET);
                let new_message_control =
                    message_control & !(1 << EXTENDED_MESSAGE_DATA_CAPABLE_SHIFT);
                self.func_handle
                    .cfg_write16(self.cap_offset + MESSAGE_CONTROL_OFFSET, new_message_control);
            }
            Ok(())
        }
    }

    pub fn set_msi_address(&mut self, address: PhysicalAddress) -> Result<(), Error> {
        if !self.is_64bit_addr_capable()
            && <PhysicalAddress as Into<u64>>::into(address) & !0xffff_ffffu64 != 0
        {
            Err(Error::AddressOutOfRange)
        } else {
            let address_low = (<PhysicalAddress as Into<u64>>::into(address) & 0xffff_ffff) as u32;
            self.func_handle.cfg_write32(self.cap_offset + MSI_ADDR_OFFSET, address_low);
            if self.is_64bit_addr_capable() {
                let address_high = (<PhysicalAddress as Into<u64>>::into(address) >> 32) as u32;
                self.func_handle.cfg_write32(self.cap_offset + MSI_ADDR_UPPER_OFFSET, address_high);
            }
            Ok(())
        }
    }
}
