//! # AArch64 TLB Maintenance
//!
//! `tlbi` operations take a virtual address shifted right by 12 and, for the ASID-qualified forms,
//! the ASID in bits [63:48]. Each batch has to be bracketed by a `dsb`/`isb` pair for the
//! invalidation to be observed.

use core::arch::asm;

use crate::cpu::isa::memory::paging::PAGE_SIZE;
use crate::cpu::scheduler::system_scheduler::SYSTEM_SCHEDULER;
use crate::memory::{AddressSpaceId, VirtualAddress};

pub fn inval_range_user(asid: AddressSpaceId, base: VirtualAddress, size: usize) {
    if let Some(hwasid) = SYSTEM_SCHEDULER.read().get_lp_scheduler().lock().asid_to_hwasid(asid) {
        let raw_base = <VirtualAddress as Into<usize>>::into(base);
        unsafe {
            asm!("dsb ishst", options(nostack, preserves_flags));
            for page in (raw_base..raw_base + size * PAGE_SIZE).step_by(PAGE_SIZE) {
                // VAE1IS takes the ASID in bits [63:48] and the VA >> 12 in bits [43:0].
                let operand = ((hwasid as u64) << 48) | (page as u64 >> 12);
                asm!(
                    "tlbi vae1is, {operand}",
                    operand = in(reg) operand,
                    options(nostack, preserves_flags),
                );
            }
            asm!("dsb ish", "isb", options(nostack, preserves_flags));
        }
    }
}

pub fn inval_asid(asid: AddressSpaceId) {
    if let Some(hwasid) = SYSTEM_SCHEDULER.read().get_lp_scheduler().lock().asid_to_hwasid(asid) {
        unsafe {
            asm!(
                "dsb ishst",
                "tlbi aside1is, {operand}",
                "dsb ish",
                "isb",
                operand = in(reg) (hwasid as u64) << 48,
                options(nostack, preserves_flags),
            );
        }
    }
}

pub fn inval_range_kernel(base: VirtualAddress, num_pages: usize) {
    let raw_base = <VirtualAddress as Into<usize>>::into(base);
    unsafe {
        asm!("dsb ishst", options(nostack, preserves_flags));
        for page in (raw_base..raw_base + num_pages * PAGE_SIZE).step_by(PAGE_SIZE) {
            // VAAE1IS invalidates the address in every ASID, which is what kernel mappings want.
            asm!(
                "tlbi vaae1is, {operand}",
                operand = in(reg) page as u64 >> 12,
                options(nostack, preserves_flags),
            );
        }
        asm!("dsb ish", "isb", options(nostack, preserves_flags));
    }
}
