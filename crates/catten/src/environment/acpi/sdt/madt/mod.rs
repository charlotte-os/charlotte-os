mod entry_types;
mod interrupt_flags;
mod index;

type GlobalSystemInterrupt = u32;

pub fn map_irq_to_gsi(irq: u8)->Result<GlobalSystemInterrupt, 
