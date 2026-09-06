use super::Error;
use crate::cpu::isa::lp::LpId;
use crate::klib::bitwise::mask_shift_read;

const VECTOR_SHIFT: u8 = 0;
const VECTOR_MASK: u64 = 0xffu64 << VECTOR_SHIFT;

const DELIVERY_MODE_SHIFT: u8 = 8;
const DELIVERY_MODE_MASK: u64 = 0b111u64 << DELIVERY_MODE_SHIFT;

const DEST_MODE_SHIFT: u8 = 11;
const DEST_MODE_MASK: u64 = 0b1u64 << DEST_MODE_SHIFT;

const DELIVERY_PENDING_SHIFT: u8 = 12;
const DELIVERY_PENDING_MASK: u64 = 0b1u64 << DELIVERY_PENDING_SHIFT;

const PIN_POLARITY_SHIFT: u8 = 13;
const PIN_POLARITY_MASK: u64 = 0b1u64 << PIN_POLARITY_SHIFT;

const TRIGGER_MODE_SHIFT: u8 = 15;
const TRIGGER_MODE_MASK: u64 = 0b1u64 << TRIGGER_MODE_SHIFT;

const MASK_SHIFT: u8 = 16;
const MASK_MASK: u64 = 0b1u64 << MASK_SHIFT;

const DESTINATION_SHIFT: u8 = 56;
const DESTINATION_MASK: u64 = 0xffu64 << DESTINATION_SHIFT;
const DESTINATION_MAX: LpId = 0xff;

#[repr(u8)]
pub enum IoApicDeliveryMode {
    Fixed = 0b000,
    LowestPriority = 0b001,
    Smi = 0b010,
    Nmi = 0b100,
    Init = 0b101,
    ExtInt = 0b111,
}

impl TryFrom<u8> for IoApicDeliveryMode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b000 => Ok(IoApicDeliveryMode::Fixed),
            0b001 => Ok(IoApicDeliveryMode::LowestPriority),
            0b010 => Ok(IoApicDeliveryMode::Smi),
            0b100 => Ok(IoApicDeliveryMode::Nmi),
            0b101 => Ok(IoApicDeliveryMode::Init),
            0b111 => Ok(IoApicDeliveryMode::ExtInt),
            _ => Err(Error::InvalidDeliveryMode(value)),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub struct IoApicRedirEntry(pub u64);

impl From<u64> for IoApicRedirEntry {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl IoApicRedirEntry {
    pub fn get_vector(&self) -> u8 {
        mask_shift_read(self.0, VECTOR_MASK, VECTOR_SHIFT) as u8
    }

    pub fn set_vector(mut self, vector: u8) -> Self {
        self.0 = (self.0 & !VECTOR_MASK) | ((vector as u64) << VECTOR_SHIFT);
        self
    }

    pub fn get_delivery_mode(&self) -> Result<IoApicDeliveryMode, Error> {
        let raw_delivery_mode =
            mask_shift_read(self.0, DELIVERY_MODE_MASK, DELIVERY_MODE_SHIFT) as u8;
        IoApicDeliveryMode::try_from(raw_delivery_mode)
    }

    pub fn set_delivery_mode(mut self, delivery_mode: IoApicDeliveryMode) -> Self {
        self.0 = (self.0 & !DELIVERY_MODE_MASK) | ((delivery_mode as u64) << DELIVERY_MODE_SHIFT);
        self
    }

    pub fn is_dest_mode_logical(&self) -> bool {
        mask_shift_read(self.0, DEST_MODE_MASK, DEST_MODE_SHIFT) != 0
    }

    pub fn set_dest_mode(mut self, is_logical: bool) -> Self {
        self.0 = (self.0 & !DEST_MODE_MASK) | ((is_logical as u64) << DEST_MODE_SHIFT);
        self
    }

    pub fn is_delivery_pending(&self) -> bool {
        mask_shift_read(self.0, DELIVERY_PENDING_MASK, DELIVERY_PENDING_SHIFT) != 0
    }

    pub fn get_pin_polarity(&self) -> bool {
        mask_shift_read(self.0, PIN_POLARITY_MASK, PIN_POLARITY_SHIFT) != 0
    }

    pub fn set_pin_polarity(mut self, is_active_low: bool) -> Self {
        self.0 = (self.0 & !PIN_POLARITY_MASK) | ((is_active_low as u64) << PIN_POLARITY_SHIFT);
        self
    }

    pub fn is_level_triggered(&self) -> bool {
        mask_shift_read(self.0, TRIGGER_MODE_MASK, TRIGGER_MODE_SHIFT) != 0
    }

    pub fn set_trigger_mode(mut self, is_level_triggered: bool) -> Self {
        self.0 =
            (self.0 & !TRIGGER_MODE_MASK) | ((is_level_triggered as u64) << TRIGGER_MODE_SHIFT);
        self
    }

    pub fn is_masked(&self) -> bool {
        mask_shift_read(self.0, MASK_MASK, MASK_SHIFT) != 0
    }

    pub fn set_mask_state(mut self, mask_state: bool) -> Self {
        self.0 = (self.0 & !MASK_MASK) | ((mask_state as u64) << MASK_SHIFT);
        self
    }

    pub fn get_destination(&self) -> LpId {
        mask_shift_read(self.0, DESTINATION_MASK, DESTINATION_SHIFT) as LpId
    }

    pub fn set_destination(mut self, destination: LpId) -> Result<Self, Error> {
        if destination > DESTINATION_MAX {
            Err(Error::LpIdOutOfRange(destination))
        } else {
            self.0 = (self.0 & !DESTINATION_MASK) | ((destination as u64) << DESTINATION_SHIFT);
            Ok(self)
        }
    }
}
