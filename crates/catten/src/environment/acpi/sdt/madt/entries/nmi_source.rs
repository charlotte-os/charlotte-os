use crate::environment::acpi::sdt::madt::GlobalSystemInterrupt;
use crate::environment::acpi::sdt::madt::entries::MadtEntryType;
use crate::environment::acpi::sdt::madt::entries::interrupt_flags::InterruptFlags;

/// The MADT NMI Source Structure overlay struct.
/// Ref: ACPI 6.6 Section 5.2.12.6
#[derive(Debug, PartialEq, Eq)]
#[repr(C, packed)]
pub struct NmiSourceEntry {
    entry_type:              MadtEntryType,
    length:                  u8,
    flags:                   InterruptFlags,
    global_system_interrupt: GlobalSystemInterrupt,
}
