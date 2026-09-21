use spin::LazyLock;

use super::INTERRUPT_STACK_SIZE;
use super::gdt::*;
use crate::cpu::isa::interrupts::fixed::register_fixed_isr_gates;
use crate::cpu::isa::interrupts::idt::Idt;
use crate::cpu::isa::lp::ops::init_lp_state;
use crate::early_logln;

static mut BSP_INTERRUPT_STACK: [u8; INTERRUPT_STACK_SIZE] = [0u8; INTERRUPT_STACK_SIZE];
static mut BSP_DF_STACK: [u8; INTERRUPT_STACK_SIZE] = [0u8; INTERRUPT_STACK_SIZE];
pub static BSP_TSS: LazyLock<Tss> = LazyLock::new(|| unsafe {
    Tss::new(
        (&raw const BSP_INTERRUPT_STACK).byte_add(INTERRUPT_STACK_SIZE) as u64,
        (&raw const BSP_DF_STACK).byte_add(INTERRUPT_STACK_SIZE) as u64,
    )
});
static BSP_GDT: LazyLock<Gdt> = LazyLock::new(|| Gdt::new(&BSP_TSS));
pub static BSP_IDT: LazyLock<Idt> = LazyLock::new(|| {
    let mut idt = Idt::new();
    register_fixed_isr_gates(&mut idt);
    crate::cpu::isa::interrupts::dynamic::register_dynamic_isr_gates(&mut idt);
    idt
});

pub fn init_bsp() {
    BSP_GDT.load();
    unsafe {
        reload_segment_regs();
    }
    BSP_IDT.load();
    init_lp_state();
    early_logln!("LP 0: x86-64 ISA initialization complete");
}
