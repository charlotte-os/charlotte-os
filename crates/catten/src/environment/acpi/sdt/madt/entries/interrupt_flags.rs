const FLAGS_POLARITY_MASK: u16 = 0b11;
const FLAGS_POLARITY_SHIFT: u16 = 0;
const FLAGS_TRIGGER_MASK: u16 = 0b11;
const FLAGS_TRIGGER_SHIFT: u16 = 2;

#[derive(Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum InterruptPolarity {
    BusSpec = 0b00,
    ActiveHigh = 0b01,
    Reserved = 0b10,
    ActiveLow = 0b11,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum InterruptTrigger {
    BusSpec = 0b00,
    Edge = 0b01,
    Reserved = 0b10,
    Level = 0b11,
}

/// The InterruptFlags struct represents the flags field in the MADT entries that specify interrupt
/// polarity and trigger mode.
/// Ref: ACPI 6.6 Section 5.2.12.5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub(super) struct InterruptFlags(u16);

impl InterruptFlags {
    pub fn polarity(&self) -> InterruptPolarity {
        match (self.0 & FLAGS_POLARITY_MASK) >> FLAGS_POLARITY_SHIFT {
            0b00 => InterruptPolarity::BusSpec,
            0b01 => InterruptPolarity::ActiveHigh,
            0b10 => InterruptPolarity::Reserved,
            0b11 => InterruptPolarity::ActiveLow,
            _ => unreachable!(),
        }
    }

    pub fn trigger(&self) -> InterruptTrigger {
        match (self.0 & FLAGS_TRIGGER_MASK) >> FLAGS_TRIGGER_SHIFT {
            0b00 => InterruptTrigger::BusSpec,
            0b01 => InterruptTrigger::Edge,
            0b10 => InterruptTrigger::Reserved,
            0b11 => InterruptTrigger::Level,
            _ => unreachable!(),
        }
    }
}
