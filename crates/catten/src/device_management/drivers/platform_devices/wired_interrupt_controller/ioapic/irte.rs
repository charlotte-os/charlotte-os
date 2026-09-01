//! # Interrupt Redirection Table Entry (IRTE)

use crate::device_management::interrupt_routing::InterruptTarget;
use crate::klib::bitwise::splice_into;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Irte(u64);

pub(super) enum DeliveryMode {
    Fixed = 0b000,
    LowestPriority = 0b001,
    SMI = 0b010,
    NMI = 0b100,
    INIT = 0b101,
    ExtINT = 0b111,
}

impl Irte {
    fn set_vector(&mut self, vector: u8) {
        const VECTOR_SHIFT: u8 = 0;
        const VECTOR_MASK: u64 = 0xff << VECTOR_SHIFT;
        splice_into(&mut self.0, vector as u64, VECTOR_MASK, VECTOR_SHIFT).unwrap();
    }

    fn set_delivery_mode(&mut self, delivery_mode: DeliveryMode) {
        const DELIVERY_MODE_SHIFT: u8 = 8;
        const DELIVERY_MODE_MASK: u64 = 0b111 << DELIVERY_MODE_SHIFT;
        splice_into(&mut self.0, delivery_mode as u64, DELIVERY_MODE_MASK, DELIVERY_MODE_SHIFT)
            .unwrap();
    }

    /*
        This function is here for completeness but it should never be used and the destination mode be should be kept cleared
        because this kernel only ever uses the local x2APIC with flat 32-bit physical destination mode.

        fn set_destination_mode(&mut self, destination_mode: bool) {
            const DESTINATION_MODE_SHIFT: u8 = 11;
            const DESTINATION_MODE_MASK: u64 = 0b1 << DESTINATION_MODE_SHIFT;
            splice_into(
                &mut self.0,
                destination_mode as u64,
                DESTINATION_MODE_MASK,
                DESTINATION_MODE_SHIFT,
            )
            .unwrap();
        }
    */

    fn set_pin_polarity(&mut self, active_low: bool) {
        const PIN_POLARITY_SHIFT: u8 = 13;
        const PIN_POLARITY_MASK: u64 = 0b1 << PIN_POLARITY_SHIFT;
        splice_into(&mut self.0, active_low as u64, PIN_POLARITY_MASK, PIN_POLARITY_SHIFT).unwrap();
    }

    fn set_trigger_mode(&mut self, latched: bool) {
        const TRIGGER_MODE_SHIFT: u8 = 15;
        const TRIGGER_MODE_MASK: u64 = 0b1 << TRIGGER_MODE_SHIFT;
        splice_into(&mut self.0, latched as u64, TRIGGER_MODE_MASK, TRIGGER_MODE_SHIFT).unwrap();
    }

    fn set_mask_bit(&mut self, mask_bit: bool) {
        const MASK_BIT_SHIFT: u8 = 16;
        const MASK_BIT_MASK: u64 = 0b1 << MASK_BIT_SHIFT;
        splice_into(&mut self.0, mask_bit as u64, MASK_BIT_MASK, MASK_BIT_SHIFT).unwrap();
    }

    fn set_dest_apic_id(&mut self, dest: IoapicDest) {
        const DEST_APIC_ID_SHIFT: u8 = 56;
        const DEST_APIC_ID_MASK: u64 = 0x0f << DEST_APIC_ID_SHIFT;
        splice_into(&mut self.0, dest.0 as u64, DEST_APIC_ID_MASK, DEST_APIC_ID_SHIFT).unwrap();
    }
}

#[repr(transparent)]
pub(super) struct IoapicDest(u8);

impl IoapicDest {
    fn try_new(apic_id: u8) -> Option<Self> {
        if apic_id < 16 {
            Some(Self(apic_id))
        } else {
            None
        }
    }
}
