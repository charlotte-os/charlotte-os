use alloc::boxed::Box;
use alloc::vec::Vec;

use spin::lazylock::LazyLock;

use super::{INTERRUPT_STACK_SIZE, gdt};
use crate::cpu::isa::interrupts::idt::{Idt, asm_load_idt};
use crate::cpu::isa::interrupts::x2apic::X2Apic;
use crate::cpu::isa::lp::ops::{get_lp_id, init_lp_state};
use crate::cpu::multiprocessor::get_lp_count;
use crate::logln;

static AP_INTERRUPT_STACKS: LazyLock<Vec<[u8; INTERRUPT_STACK_SIZE]>> = LazyLock::new(|| {
    logln!("LP {}: Computing the number of AP interrupt stacks to allocate.", (get_lp_id()));
    let num_aps = get_lp_count() - 1; // Exclude BSP
    logln!("LP {}: Allocating {} AP interrupt stacks.", (get_lp_id()), num_aps);
    let mut ret = Vec::<[u8; INTERRUPT_STACK_SIZE]>::with_capacity(num_aps as usize);
    for _ in 0..num_aps {
        ret.push(*(Box::new([0u8; INTERRUPT_STACK_SIZE])));
    }
    logln!("LP {}: AP interrupt stacks allocated.", (get_lp_id()));
    ret
});

static AP_DF_STACKS: LazyLock<Vec<[u8; INTERRUPT_STACK_SIZE]>> = LazyLock::new(|| {
    logln!("LP {}: Computing the number of AP double fault stacks to allocate.", (get_lp_id()));
    let num_aps = get_lp_count() - 1; // Exclude BSP
    logln!("LP {}: Allocating {} AP df stacks.", (get_lp_id()), num_aps);
    let mut ret = Vec::<[u8; INTERRUPT_STACK_SIZE]>::with_capacity(num_aps as usize);
    for _ in 0..num_aps {
        ret.push(*(Box::new([0u8; INTERRUPT_STACK_SIZE])));
    }
    logln!("LP {}: AP df stacks allocated.", (get_lp_id()));
    ret
});

pub static AP_TSS: LazyLock<Vec<super::gdt::Tss>> = LazyLock::new(|| {
    logln!("LP {}: Creating the TSS vector.", (get_lp_id()));
    let mut tsses = Vec::new();
    logln!("LP {}: Allocating {} TSS entries.", (get_lp_id()), (get_lp_count() - 1));
    for i in 0..(get_lp_count() - 1) {
        tsses.push(super::gdt::Tss::new(
            unsafe { (&raw const AP_INTERRUPT_STACKS[i as usize]).byte_add(INTERRUPT_STACK_SIZE) }
                as u64,
            unsafe { (&raw const AP_DF_STACKS[i as usize]).byte_add(INTERRUPT_STACK_SIZE) } as u64,
        ));
    }
    logln!("LP {}: TSS vector initialized.", (get_lp_id()));
    tsses
});

static AP_GDTS: LazyLock<Vec<super::gdt::Gdt>> = LazyLock::new(|| {
    logln!("LP {}: Creating the GDT vector.", (get_lp_id()));
    let mut gdts = Vec::new();
    logln!("LP {}: Allocating {} GDT entries.", (get_lp_id()), (get_lp_count() - 1));
    for tss in AP_TSS.iter() {
        logln!("LP {}: Constructing and allocating a GDT", (get_lp_id()));
        gdts.push(super::gdt::Gdt::new(tss));
    }
    logln!("LP {}: GDT vector initialized.", (get_lp_id()));
    gdts
});

pub static AP_IDTS: LazyLock<Vec<crate::cpu::isa::interrupts::idt::Idt>> = LazyLock::new(|| {
    logln!("LP {}: Creating the IDT vector.", (get_lp_id()));
    let mut idts = Vec::new();
    logln!("LP {}: Allocating {} IDT entries.", (get_lp_id()), (get_lp_count() - 1));
    for _ in 0..(get_lp_count() - 1) {
        logln!("LP {}: Constructing and allocating an IDT", (get_lp_id()));
        let mut idt = crate::cpu::isa::interrupts::idt::Idt::new();
        logln!("LP {}: Registering fixed interrupt gates.", (get_lp_id()));
        crate::cpu::isa::interrupts::fixed::register_fixed_isr_gates(&mut idt);
        crate::cpu::isa::interrupts::dynamic::register_dynamic_isr_gates(&mut idt);
        logln!("LP {}: Pushing the initialized IDT to the vector.", (get_lp_id()));
        idts.push(idt);
    }
    logln!("LP {}: IDT vector initialized.", (get_lp_id()));
    idts
});

pub static AP_IDTRS: LazyLock<Vec<crate::cpu::isa::interrupts::idt::Idtr>> = LazyLock::new(|| {
    logln!("LP {}: Creating the IDTR vector.", (get_lp_id()));
    let mut idtrs = Vec::new();
    logln!("LP {}: Allocating {} IDTR entries.", (get_lp_id()), (get_lp_count() - 1));
    for idt in AP_IDTS.iter() {
        logln!("LP {}: Constructing and allocating an IDTR", (get_lp_id()));
        idtrs.push(crate::cpu::isa::interrupts::idt::Idtr::new(
            (size_of::<Idt>() - 1) as u16,
            idt as *const Idt as u64,
        ));
    }
    logln!("LP {}: IDTR vector initialized.", (get_lp_id()));
    idtrs
});

pub fn init_ap() {
    let lp_id = crate::cpu::isa::lp::ops::get_lp_id();
    logln!("LP {}: Computing AP index.", lp_id);
    let ap_index = (lp_id - 1) as usize; // APs start from LP1
    logln!("LP {}: AP index is {}.", lp_id, ap_index);
    crate::logln!("LP {}: Initializing TSS, GDT, and IDT", lp_id);
    AP_GDTS[ap_index].load();
    unsafe {
        gdt::reload_segment_regs();
    }
    unsafe { asm_load_idt(&raw const AP_IDTRS[ap_index]) };
    init_lp_state();
    X2Apic::record_id();
    crate::logln!("LP {}: x86-64 logical processor initialization complete", lp_id);
}
