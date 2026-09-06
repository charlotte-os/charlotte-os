use crate::environment::acpi::sdt::madt::GlobalSystemInterrupt;
use crate::environment::acpi::sdt::madt::entries::MadtEntryType;
use crate::environment::acpi::sdt::madt::entries::interrupt_flags::InterruptFlags;

/// The MADT Interrupt Source Override Structure overlay struct.
/// Ref: ACPI 6.6 Section 5.2.12.5
pub struct InterruptSourceOverrideEntry {
    pub entry_type: MadtEntryType,
    pub length: u8,
    // Always 0 meaning the ISA bus per ACPI 6.6
    pub bus: u8,
    pub irq_source: u8,
    pub global_system_interrupt: GlobalSystemInterrupt,
    pub flags: InterruptFlags,
}
