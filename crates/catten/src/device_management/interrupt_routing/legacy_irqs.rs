use spin::LazyLock;

use crate::cpu::isa::types::WiredIntCtlr;
use crate::device_management::interrupt_routing::WiredSource;
use crate::environment::acpi::sdt::madt::interface::{IrqGsiMapping, create_irq_gsi_mapping_table};

pub const LEGACY_IRQ_COUNT: usize = 16;

static LEGACY_IRQ_TO_GSI_MAPPING: LazyLock<[IrqGsiMapping; LEGACY_IRQ_COUNT]> =
    LazyLock::new(create_irq_gsi_mapping_table);

fn legacy_irq_to_wired_source(legacy_irq: u8) -> WiredSource {
    let gsi_num = if let Some(mapping) = LEGACY_IRQ_TO_GSI_MAPPING.get(legacy_irq as usize) {
        mapping.gsi
    } else {
        legacy_irq as u32
    };
    todo!(
        "Go through the IOAPIC list to find which IOAPIC number and pin to use to fill out the \
         WiredSource struct."
    )
}
