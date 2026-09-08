mod entries;

use entries::interrupt_source_override::InterruptSourceOverrideEntry;

use crate::device_management::interrupt_routing::legacy_irqs::LEGACY_IRQ_COUNT;
use crate::device_management::interrupt_routing::{InterruptSource, WiredSource};
use crate::environment::acpi::Error;
use crate::environment::acpi::sdt::madt::entries::MADT_INDEX;

type GlobalSystemInterrupt = u32;

#[derive(Debug)]
pub struct IrqGsiMapping {
    pub irq: u8,
    pub gsi: GlobalSystemInterrupt,
    pub polarity: bool,
    pub latched: bool,
}

/// Create a mapping table from legacy IRQ numbers to ACPI Global System Interrupts (GSIs)
pub fn create_irq_gsi_mapping_table() -> [IrqGsiMapping; LEGACY_IRQ_COUNT] {
    // Set legacy defaults
    let mut irq_override_table = core::array::from_fn(|irq| IrqGsiMapping {
        irq: irq as u8,
        gsi: irq as GlobalSystemInterrupt,
        polarity: true,
        latched: false,
    });
    // Override with entries from the MADT
    let irq_override_entries =
        (*MADT_INDEX).get_entries_with_type(entries::MadtEntryType::InterruptSourceOverride);
    for entry in irq_override_entries.iter() {
        let override_entry =
            unsafe { core::mem::transmute::<_, &'static InterruptSourceOverrideEntry>(entry) };
        let irq = override_entry.irq_source as usize;
        if irq < LEGACY_IRQ_COUNT {
            irq_override_table[irq].gsi = override_entry.global_system_interrupt;
            irq_override_table[irq].polarity = override_entry.flags.polarity()
                != entries::interrupt_flags::InterruptPolarity::ActiveLow;
            irq_override_table[irq].latched = override_entry.flags.trigger_mode()
                == entries::interrupt_flags::InterruptTriggerMode::Level;
        }
    }
    // Return the final IRQ to GSI mapping table
    irq_override_table
}
