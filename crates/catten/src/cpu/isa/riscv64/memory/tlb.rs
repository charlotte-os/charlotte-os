//! # RISC-V TLB Maintenance
//!
//! `sfence.vma` takes an optional virtual address in `rs1` and an optional ASID in `rs2`; passing
//! `x0` for either widens the invalidation to cover everything in that dimension.

use core::arch::asm;

use crate::cpu::isa::memory::paging::PAGE_SIZE;
use crate::cpu::scheduler::system_scheduler::SYSTEM_SCHEDULER;
use crate::memory::{AddressSpaceId, VirtualAddress};

pub fn inval_range_user(asid: AddressSpaceId, base: VirtualAddress, size: usize) {
    if let Some(hwasid) = SYSTEM_SCHEDULER.read().get_lp_scheduler().lock().asid_to_hwasid(asid) {
        let raw_base = <VirtualAddress as Into<usize>>::into(base);
        for page in (raw_base..raw_base + size * PAGE_SIZE).step_by(PAGE_SIZE) {
            unsafe {
                asm!(
                    "sfence.vma {page}, {asid}",
                    page = in(reg) page,
                    asid = in(reg) hwasid as usize,
                    options(nostack, preserves_flags),
                );
            }
        }
    }
}

pub fn inval_asid(asid: AddressSpaceId) {
    if let Some(hwasid) = SYSTEM_SCHEDULER.read().get_lp_scheduler().lock().asid_to_hwasid(asid) {
        unsafe {
            asm!(
                "sfence.vma x0, {asid}",
                asid = in(reg) hwasid as usize,
                options(nostack, preserves_flags),
            );
        }
    }
}

pub fn inval_range_kernel(base: VirtualAddress, num_pages: usize) {
    let raw_base = <VirtualAddress as Into<usize>>::into(base);
    for page in (raw_base..raw_base + num_pages * PAGE_SIZE).step_by(PAGE_SIZE) {
        unsafe {
            asm!(
                "sfence.vma {page}, x0",
                page = in(reg) page,
                options(nostack, preserves_flags),
            );
        }
    }
}
