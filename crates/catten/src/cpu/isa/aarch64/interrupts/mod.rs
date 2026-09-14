//! # AArch64 Interrupt Management

pub mod dynamic;
pub mod gic;

use core::arch::{asm, global_asm};

pub use gic::LocalIntCtlr;

// Include the interrupt vector table assembly
global_asm!(include_str!("ivt.asm"));

#[derive(Debug)]
pub enum Error {
    InvalidLpId,
    IntVecUnassigned(u8),
    ArgIsFixedIntVec(u8),
}

#[inline(always)]
pub fn load_ivt() {
    // Load the interrupt vector table
    unsafe {
        asm!(
            "ldr x0, =ivt",
            "msr vbar_el1, x0",
            out("x0") _,
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sync_dispatcher() {
    // The Exception State Register EL1 (ESR_EL1) holds information about the exception that
    // occurred
    let esr_el1: u64;
    unsafe {
        asm!("mrs {}, esr_el1", out(reg) esr_el1);
    }
    early_logln!("LP {}:Synchronous exception occurred with ESR_EL1 = {:#x}", get_lp_id(), esr_el1);
    const EC_NUM_BITS: u64 = 6;
    const EC_SHIFT: u64 = 26;
    let exception_class = bitwise::mask_shift_read(esr_el1, (1 << EC_NUM_BITS) - 1, EC_SHIFT);
    early_logln!("Exception class = {:#x}", exception_class);
}
#[unsafe(no_mangle)]
pub extern "C" fn irq_dispatcher() {}
#[unsafe(no_mangle)]
pub extern "C" fn fiq_dispatcher() {}
#[unsafe(no_mangle)]
pub extern "C" fn serr_dispatcher() {}
