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
pub extern "C" fn sync_dispatcher() {}
#[unsafe(no_mangle)]
pub extern "C" fn irq_dispatcher() {}
#[unsafe(no_mangle)]
pub extern "C" fn fiq_dispatcher() {}
#[unsafe(no_mangle)]
pub extern "C" fn serr_dispatcher() {}
