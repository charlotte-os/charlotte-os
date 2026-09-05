use crate::environment::acpi::sdt::madt::interrupt_flags::InterruptFlags;
use crate::environment::acpi::sdt::madt::{GlobalSystemInterrupt, MadtEntryType};

/// The MADT Interrupt Source Override Structure overlay struct.
/// Ref: ACPI 6.6 Section 5.2.12.5
pub struct InterruptSourceOverrideEntry {
    entry_type: MadtEntryType,
    length: u8,
    // Always 0 meaning the ISA bus per ACPI 6.6
    bus: u8,
    irq_source: u8,
    global_system_interrupt: GlobalSystemInterrupt,
    flags: InterruptFlags,
}
