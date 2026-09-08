use spin::LazyLock;

use crate::environment::acpi::sdt::madt::{IrqGsiMapping, create_irq_gsi_mapping_table};

pub const LEGACY_IRQ_COUNT: usize = 16;

static LEGACY_IRQ_TO_GSI_MAPPING: LazyLock<[IrqGsiMapping; LEGACY_IRQ_COUNT]> =
    LazyLock::new(create_irq_gsi_mapping_table);
