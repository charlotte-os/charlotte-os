mod entries;

use entries::interrupt_source_override::InterruptSourceOverrideEntry;
use crate::{device_management::interrupt_routing::InterruptSource, environment::acpi::{Error, sdt::madt::entries::MADT_INDEX}};

type GlobalSystemInterrupt = u32;

/// Get the ACPI GSI corresponding to the legacy ISA bus interrupt request number passes an argument
pub fn map_irq_to_gsi(irq: u8)->Result<GlobalSystemInterrupt, Error> {
    if irq > 15 {
        return Err(Error::IrqValOutOfRange);
    }
    let override_entries = (*MADT_INDEX).get_entries_with_type(entries::MadtEntryType::InterruptSourceOverride);
    for entry in override_entries.iter() {
        let override_entry = unsafe { core::mem::transmute::<_, &'static InterruptSourceOverrideEntry>(entry) };
        if override_entry.irq_source == irq {
            return Ok(override_entry.global_system_interrupt)
        }
    };
    Ok(irq as GlobalSystemInterrupt)
}

fn map_gic_to_ext_int_src(gic: GlobalSystemInterrupt)-> InterruptSource {
    todo!("Depending on the current ISA, use the MADT IOAPIC, GICD, or APLIC entries to a given ACPI GSI to an external interrupt controller and its
        specific input source number.")
}
