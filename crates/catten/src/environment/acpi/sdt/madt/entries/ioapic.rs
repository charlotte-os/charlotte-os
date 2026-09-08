use crate::environment::acpi::sdt::madt::GlobalSystemInterrupt;
use crate::environment::acpi::sdt::madt::entries::MadtEntryType;

type IoApicId = u8;

/// The MADT I/O APIC Structure overlay struct.
/// Ref: ACPI 6.6 Section 5.2.12.3
#[derive(Debug)]
#[repr(C, packed)]
pub struct IoApicEntry {
    pub entry_type: MadtEntryType,
    pub length: u8,
    pub ioapic_id: IoApicId,
    pub reserved: u8,
    pub ioapic_address: u32,
    pub global_system_interrupt_base: GlobalSystemInterrupt,
}
