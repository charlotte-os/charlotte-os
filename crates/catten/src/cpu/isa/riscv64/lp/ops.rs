//! # RISCV64 Hart (Logical Processor) Operations
//!
//! The interrupt-state and wait-for-interrupt primitives are real: they are a handful of CSR
//! instructions and there is nothing to defer. Everything that needs the scheduler, a thread
//! context or a per-hart control block is stubbed until the RISC-V port grows those pieces.

use core::arch::asm;

use super::LpId;
use crate::memory::VirtualAddress;

/// Supervisor Interrupt Enable bit in `sstatus`.
const SSTATUS_SIE: usize = 1 << 1;

pub fn init_lp_state() {
    todo!("Set up the RISC-V per-hart state: sscratch, the trap vector and the FPU context.")
}

#[rustfmt::skip]
#[macro_export]
macro_rules! await_interrupt {
    () => {
        loop {
            unsafe {
                core::arch::asm!(
                    "csrsi sstatus, 2",
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
            core::arch::asm!("csrci sstatus, 2", options(nomem, nostack));
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
            core::arch::asm!("csrsi sstatus, 2", options(nomem, nostack));
        }
    };
}
#[rustfmt::skip]
pub use unmask_interrupts;

#[inline(always)]
pub fn get_int_state() -> bool {
    let sstatus: usize;
    unsafe {
        asm!(
            "csrr {}, sstatus",
            out(reg) sstatus,
            options(nomem, nostack, preserves_flags)
        );
    }
    sstatus & SSTATUS_SIE != 0
}

pub fn get_lic_id() -> u32 {
    todo!("Report the hart's interrupt controller context once the PLIC/APLIC driver exists.")
}

pub fn store_lp_id(_id: LpId) {
    todo!("Stash the hart ID in the per-hart control block reached through sscratch.")
}

pub fn get_lp_id() -> LpId {
    todo!("Read the hart ID back from the per-hart control block reached through sscratch.")
}

#[inline]
pub extern "C" fn get_lp_local_base() -> VirtualAddress {
    todo!("Return the per-hart local storage base; RISC-V conventionally keeps it in tp.")
}

#[inline]
pub extern "C" fn set_lp_local_base(_base: VirtualAddress) {
    todo!("Set the per-hart local storage base; RISC-V conventionally keeps it in tp.")
}

#[unsafe(no_mangle)]
pub extern "C" fn cond_yield_lp() {
    todo!("Port the context-switch decision in the x86_64 cond_yield_lp to RISC-V.")
}

#[unsafe(no_mangle)]
pub extern "C" fn switch_ctx(_curr_sp_ptr: *mut u64, _next_sp_ptr: *const u64) {
    todo!("Save and restore the RISC-V callee-saved registers, satp and sstatus.")
}

#[unsafe(no_mangle)]
pub extern "C" fn enter_init_thread_ctx(_sp_ptr: *const u64) {
    todo!("Load the first thread context on this hart without saving an outgoing one.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_trampoline() -> ! {
    todo!("Enter U-mode via sret with the frame prepared by create_user_thread_context.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_thread_trampoline() -> ! {
    todo!("Enable interrupts and call the kernel thread entry point held in a saved register.")
}

/* PCIe ECAM Configuration Access

These mirror the x86_64 accessors, which use dedicated instructions to guarantee that every access
reaches the device exactly once, in order, and has completed before the next one begins.
*/

/// Read a 8-bit value from the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to a 8-bit register in a PCIe ECAM.
pub unsafe fn config_read8(_addr: VirtualAddress) -> u8 {
    todo!("Implement RISC-V PCIe ECAM reads ordered with FENCE instructions.")
}

/// Read a 16-bit value from the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to a 16-bit register in a PCIe ECAM.
pub unsafe fn config_read16(_addr: VirtualAddress) -> u16 {
    todo!("Implement RISC-V PCIe ECAM reads ordered with FENCE instructions.")
}

/// Read a 32-bit value from the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to a 32-bit register in a PCIe ECAM.
pub unsafe fn config_read32(_addr: VirtualAddress) -> u32 {
    todo!("Implement RISC-V PCIe ECAM reads ordered with FENCE instructions.")
}

/// Write a 8-bit value to the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to a 8-bit register in a PCIe ECAM.
pub unsafe fn config_write8(_addr: VirtualAddress, _value: u8) {
    todo!("Implement RISC-V PCIe ECAM writes ordered with FENCE instructions.")
}

/// Write a 16-bit value to the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to a 16-bit register in a PCIe ECAM.
pub unsafe fn config_write16(_addr: VirtualAddress, _value: u16) {
    todo!("Implement RISC-V PCIe ECAM writes ordered with FENCE instructions.")
}

/// Write a 32-bit value to the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to a 32-bit register in a PCIe ECAM.
pub unsafe fn config_write32(_addr: VirtualAddress, _value: u32) {
    todo!("Implement RISC-V PCIe ECAM writes ordered with FENCE instructions.")
}
