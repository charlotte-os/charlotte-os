mod entries;

use entries::interrupt_source_override::InterruptSourceOverrideEntry;

use crate::device_management::interrupt_routing::{InterruptSource, WiredSource};
use crate::environment::acpi::Error;
use crate::environment::acpi::sdt::madt::entries::MADT_INDEX;

type GlobalSystemInterrupt = u32;

/// Get the ACPI GSI corresponding to the legacy ISA bus interrupt request number passes an argument
pub fn get_irq_override_info(irq: u8) -> Result<WiredSource, Error> {
    if irq > 15 {
        return Err(Error::IrqValOutOfRange);
    }
    let override_entries =
        (*MADT_INDEX).get_entries_with_type(entries::MadtEntryType::InterruptSourceOverride);
    for entry in override_entries.iter() {
        let override_entry =
            unsafe { core::mem::transmute::<_, &'static InterruptSourceOverrideEntry>(entry) };
        if override_entry.irq_source == irq {
            let gic_tgt = map_gic_to_ext_int_src(override_entry.global_system_interrupt)?;
            return Ok(WiredSource {
                wired_ic_id: gic_tgt.wired_ic_id,
                source_num: gic_tgt.source_num,
                polarity: override_entry.flags.polarity()
                    != entries::interrupt_flags::InterruptPolarity::ActiveLow,
                latched: override_entry.flags.trigger_mode()
                    == entries::interrupt_flags::InterruptTriggerMode::Level,
            });
        }
    }
    Ok(map_gic_to_ext_int_src(irq as GlobalSystemInterrupt)?)
}

fn map_gic_to_ext_int_src(gic: GlobalSystemInterrupt) -> Result<WiredSource, Error> {
    todo!(
        "Depending on the current ISA, use the MADT IOAPIC, GICD, or APLIC entries to a given \
         ACPI GSI to an external interrupt controller and its
        specific input source number."
    )
}
