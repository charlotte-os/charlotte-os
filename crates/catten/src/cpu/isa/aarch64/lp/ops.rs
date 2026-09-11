//! # Low-level operations for AArch64 Logical Processors
//!
//! The DAIF and system-register accesses here are real; everything that needs the scheduler or a
//! thread context is stubbed until the AArch64 port grows those pieces.

use core::arch::asm;

use crate::cpu::isa::lp::LpId;
use crate::memory::VirtualAddress;

/// PSTATE.DAIF IRQ mask bit. Interrupts are enabled when it is clear.
const DAIF_IRQ_MASK: u64 = 1 << 7;

pub fn init_lp_state() {
    todo!("Set up the AArch64 per-LP state: TPIDR_EL1, the exception stacks and the FPU context.")
}

#[rustfmt::skip]
#[macro_export]
macro_rules! await_interrupt {
    () => {
        loop {
            unsafe {
                core::arch::asm!(
                    "msr daifclr, 0b0011",
                    "wfi",
                    options(nomem, nostack, preserves_flags)
                );
            }
        }
    };
}
#[rustfmt::skip]
pub use await_interrupt;

#[rustfmt::skip]
#[macro_export]
macro_rules! mask_interrupts {
    () => {
        unsafe {
            core::arch::asm!("msr daifset, 0b0011", options(nomem, nostack));
        }
    };
}
#[rustfmt::skip]
pub use mask_interrupts;

#[rustfmt::skip]
#[macro_export]
macro_rules! unmask_interrupts {
    () => {
        unsafe {
            core::arch::asm!("msr daifclr, 0b0011", options(nomem, nostack));
        }
    };
}
#[rustfmt::skip]
pub use unmask_interrupts;

#[inline(always)]
pub fn get_int_state() -> bool {
    let daif: u64;
    unsafe {
        asm!(
            "mrs {}, daif",
            out(reg) daif,
            options(nomem, nostack, preserves_flags)
        );
    }
    daif & DAIF_IRQ_MASK == 0
}

pub fn store_lp_id(lp_id: LpId) {
    unsafe {
        asm!(
            "msr tpidr_el1, {lp_id:x}",
            lp_id = in(reg) lp_id as u64,
            options(nomem, nostack, preserves_flags)
        );
    }
}

pub fn get_lp_id() -> LpId {
    let lp_id: u64;
    unsafe {
        asm!(
            "mrs {lp_id}, tpidr_el1",
            lp_id = out(reg) lp_id,
            options(nomem, nostack, preserves_flags)
        );
    }
    lp_id as LpId
}

pub fn get_lic_id() -> u32 {
    let mpidr_el1: u64;
    unsafe {
        asm!(
            "mrs {mpidr_el1}, mpidr_el1",
            mpidr_el1 = out(reg) mpidr_el1,
            options(nomem, nostack, preserves_flags)
        );
    }
    // The Affinity Level 0 field (bits [7:0]) contains the CPU ID within the cluster
    (mpidr_el1 & 0xff) as u32
}

#[inline]
pub extern "C" fn set_lp_local_base(vaddr: VirtualAddress) {
    unsafe {
        asm!(
            "msr tpidr_el0, {vaddr:x}",
            vaddr = in(reg) <VirtualAddress as Into<usize>>::into(vaddr) as u64,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline]
pub extern "C" fn get_lp_local_base() -> VirtualAddress {
    let addr: u64;
    unsafe {
        asm!(
            "mrs {addr}, tpidr_el0",
            addr = out(reg) addr,
            options(nomem, nostack, preserves_flags)
        );
    }
    VirtualAddress::from(addr as usize)
}

#[unsafe(no_mangle)]
pub extern "C" fn cond_yield_lp() {
    todo!("Port the context-switch decision in the x86_64 cond_yield_lp to AArch64.")
}

#[unsafe(no_mangle)]
pub extern "C" fn switch_ctx(_curr_sp_ptr: *mut u64, _next_sp_ptr: *const u64) {
    todo!("Save and restore x19-x30, the stack pointer, TTBR0_EL1 and SPSR_EL1.")
}

#[unsafe(no_mangle)]
pub extern "C" fn enter_init_thread_ctx(_sp_ptr: *const u64) {
    todo!("Load the first thread context on this LP without saving an outgoing one.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_trampoline() -> ! {
    todo!("Enter EL0 via eret with the frame prepared by create_user_thread_context.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_thread_trampoline() -> ! {
    todo!("Unmask interrupts and call the kernel thread entry point held in a saved register.")
}
