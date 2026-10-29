use core::arch::asm;

use crate::cpu::isa::memory::paging::PAGE_SIZE;
use crate::memory::VirtualAddress;

pub fn inval_range(base: VirtualAddress, num_pages: usize) {
    let raw_base = <VirtualAddress as Into<usize>>::into(base);
    let len_bytes = num_pages * PAGE_SIZE;
    for page in (raw_base..raw_base + len_bytes).step_by(PAGE_SIZE) {
        unsafe {
            asm!(
                "invlpg [{page}]",
                page = in(reg) page,
                options(nostack, preserves_flags),
            );
        }
    }
}
