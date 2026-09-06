use crate::environment::acpi::sdt::madt::entries::MadtEntryType;
use crate::environment::acpi::sdt::madt::entries::interrupt_flags::InterruptFlags;

/// The MADT Local x2APIC NMI Structure overlay struct
/// The entries specified by this structure need to raise an NMI on the host processor (vector 2)
/// Ref: ACPI 6.6 Section 5.2.12.13
pub struct LocalX2ApicNmiEntry {
    entry_type:            MadtEntryType,
    entry_length:          u8,
    flags:                 InterruptFlags,
    acpi_proc_uid:         u32,
    local_interrupt_entry: u8,
}
