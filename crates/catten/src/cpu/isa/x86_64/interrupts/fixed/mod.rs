//! ISRs with fixed vector numbers across all logical processors

pub mod exceptions;
pub mod ipis;
pub mod spurious;
pub mod timer;

use crate::cpu::isa::constants::interrupt_vectors::{
    IPI_DOORBELL_VECTOR,
    LAPIC_TIMER_VECTOR,
    SPURIOUS_INTERRUPT_VECTOR_NUM,
};
use crate::cpu::isa::init::gdt::KERNEL_CODE_SELECTOR;
use crate::cpu::isa::interrupts::idt::Idt;

pub fn register_fixed_isr_gates(idt: &mut Idt) {
    exceptions::set_gates(idt);
    idt.set_gate(
        SPURIOUS_INTERRUPT_VECTOR_NUM,
        spurious::isr_spurious,
        KERNEL_CODE_SELECTOR,
        None,
        false,
        true,
    );
    idt.set_gate(
        LAPIC_TIMER_VECTOR,
        timer::isr_lapic_timer,
        KERNEL_CODE_SELECTOR,
        None,
        false,
        true,
    );
    idt.set_gate(
        IPI_DOORBELL_VECTOR,
        ipis::isr_ipi_doorbell,
        KERNEL_CODE_SELECTOR,
        None,
        false,
        true,
    );
}
