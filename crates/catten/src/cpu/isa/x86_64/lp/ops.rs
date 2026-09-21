//! # Low-level operations for x86_64 Logical Processors

pub fn init_lp_state() {
    unsafe {
        core::arch::asm! {
            "mov rax, cr4",
            "or rax, 1<<16", // Set the bit to enable FSGSBASE instructions
            "mov cr4, rax",
            "rdtscp", // Read the kernel assigned processor ID from the TSC_AUX register
            "wrfsbase rcx",
            "wrgsbase rcx",
            out("rax") _,
            out("rcx") _,
            out("rdx") _,
        }
    }
}

#[rustfmt::skip]
#[macro_export]
macro_rules! await_interrupt {
    () => {
        loop {
            unsafe {
                core::arch::asm!(
                    "sti",
                    "hlt", 
                    options(nomem, nostack, preserves_flags)
                );
            }
        }
    };
}
#[rustfmt::skip]
pub use await_interrupt;

#[inline(always)]
pub fn get_int_state() -> bool {
    let rflags: u64;
    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {}",
            out(reg) rflags,
            options(nomem, nostack, preserves_flags)
        );
    }
    rflags & (1 << rflags::INTERRUPT_FLAG_SHIFT) != 0
}

#[rustfmt::skip]
#[macro_export]
macro_rules! mask_interrupts {
    () => {
        unsafe {
            core::arch::asm!("cli", options(nomem, nostack));
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
            core::arch::asm!("sti", options(nomem, nostack));
        }
    };
}
#[rustfmt::skip]
pub use unmask_interrupts;

pub fn get_lic_id() -> u32 {
    let apic_id: u32;
    use crate::cpu::isa::constants::*;
    unsafe {
        core::arch::asm!(
            "rdmsr",
            inlateout("ecx") msrs::x2apic::ID_REG => _,
            lateout("eax") apic_id,
            lateout("edx") _,
            options(nostack, preserves_flags)
        );
    }
    apic_id
}

use core::arch::{asm, naked_asm};

use super::LpId;
use crate::cpu::isa::constants::*;

pub fn store_lp_id(id: LpId) {
    let id_upper = ((id as u64) >> 32) as u32;
    let id_lower = ((id as u64) & (1 << 32) - 1) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("eax") id_lower,
            in("edx") id_upper,
            in("ecx") msrs::TSC_AUX,
            options(nostack, preserves_flags)
        );
    }
}

pub fn get_lp_id() -> LpId {
    let mut id: u32;
    unsafe {
        core::arch::asm!(
            "rdtscp",
            out("edx") _,
            out("eax") _,
            out("ecx") id,
        );
    }
    id as crate::cpu::isa::lp::LpId
}

use crate::cpu::scheduler::system_scheduler::SYSTEM_SCHEDULER;
use crate::cpu::scheduler::threads::MASTER_THREAD_TABLE;
use crate::logln;
use crate::memory::VirtualAddress;

#[inline]
pub extern "C" fn get_lp_local_base() -> VirtualAddress {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "rdgsbase {}",
            out(reg) ret,
            options(nomem, nostack, preserves_flags)
        );
    }
    VirtualAddress::from(ret)
}

#[inline]
pub extern "C" fn set_lp_local_base(base: VirtualAddress) {
    unsafe {
        core::arch::asm!(
            "wrgsbase {}",
            in(reg) <VirtualAddress as Into<u64>>::into(base),
            options(nomem, nostack, preserves_flags)
        )
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cond_yield_lp() {
    let interrupts_were_enabled = get_int_state();
    mask_interrupts!();
    loop {
        // No scheduler/table guard may survive a switch or the idle instruction.
        let mut idle = false;
        let switch_params = {
            let sched = SYSTEM_SCHEDULER.read();
            let mut local = sched.get_lp_scheduler().lock();
            if !local.is_ctx_switch_pending() {
                None
            } else {
                let current = local.get_tid();
                match local.next() {
                    Ok(next) => {
                        local.clear_ctx_switch_pending();
                        if current == Some(next) {
                            None
                        } else {
                            let mut threads = MASTER_THREAD_TABLE.write();
                            let saved_stack = current.map_or(core::ptr::null_mut(), |tid| {
                                &raw mut threads
                                    .get_mut(tid)
                                    .unwrap()
                                    .context
                                    .kernel_stack_buf
                                    .curr_sp
                            });
                            let next_stack = &raw const threads
                                .get(next)
                                .unwrap()
                                .context
                                .kernel_stack_buf
                                .curr_sp;
                            Some((saved_stack, next_stack))
                        }
                    }
                    Err(_) => {
                        local.stop();
                        idle = true;
                        None
                    }
                }
            }
        };
        if let Some((current, next)) = switch_params {
            switch_ctx(current.cast::<u64>(), next.cast::<u64>());
        }
        if !idle {
            break;
        }
        // STI's interrupt shadow closes the wakeup-before-HLT race. The blocked
        // thread's current handle remains available until its stack is saved.
        unsafe {
            asm!("sti", "hlt", "cli", options(nomem, nostack));
        }
    }
    if interrupts_were_enabled {
        unmask_interrupts!();
    }
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "C" fn switch_ctx(curr_rsp0_ptr: *mut u64, next_rsp0_ptr: *const u64) {
    naked_asm!(
        // if `curr_rsp0_ptr` is null, then we are yielding from a non-thread context (e.g., the initial kernel thread context after boot) and thus we don't need to save the current context
        "cmp rdi, 0",
        "je skip_save",
        // save caller-saved registers
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "pushfq",
        "mov rax, cr3",
        "push rax",
        // compute the stack pointer offset in the thread context and save it to the current thread context
        "mov [rdi], rsp",
        "skip_save:",
        // load the stack pointer from the next thread context
        "mov rsp, [rsi]",
        // restore caller-saved registers
        "pop rax",
        "mov cr3, rax",
        "pop rax",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        "push rax",
        "popfq",
        // return to the next thread
        "ret",
    );
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub extern "C" fn enter_init_thread_ctx(rsp0_ptr: *const u64) {
    naked_asm!(
                // load the stack pointer from the next thread context
        "mov rsp, [rsi]",
        // restore caller-saved registers
        "pop rax",
        "mov cr3, rax",
        "popfq",
        "xor r15, r15",
        "xor r14, r14",
        "xor r13, r13",
        "xor r12, r12",
        "xor r11, r11",
        "xor r10, r10",
        "xor r9, r9",
        "xor r8, r8",
        "xor rbp, rbp",
        "xor rdx, rdx",
        "xor rcx, rcx",
        "xor rbx, rbx",
        "xor rax, rax",     
        // return to the thread's kernel entry point (which will then `iretq` to the user entry point for user threads)
        "ret",
    );
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn user_trampoline() -> ! {
    // Safety: This function should only be entered by returning from `yield_lp` after having
    // switched to a new user thread. The caller is responsible for ensuring that the stack is
    // properly set up with a `UserEntryFrames` struct, and that the CPU is in the correct state for
    // executing this trampoline (e.g., interrupts disabled, correct segment selectors, etc.).
    naked_asm!(
        // Set the current stack pointer as the one to use for the kernel stack in the TSS
        "mov rdi, rsp",
        "call update_tss_rsp0",
        // Switch to the userspace value for GS.BASE, saving the kernel value in the MSR
        "swapgs",
        // `iretq` to the user entry point
        "iretq",
    );
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn kernel_thread_trampoline() -> ! {
    naked_asm!(
        "sti",
        "sub rsp, 8",
        "call r12",
        "call finish_kernel_thread",
        "ud2",
    );
}

/// A completed kernel thread parks permanently. Its stack is retained because
/// reclaiming the stack currently executing this function would be unsafe.
#[unsafe(no_mangle)]
pub extern "C" fn finish_kernel_thread() -> ! {
    let tid = crate::cpu::scheduler::system_scheduler::get_thread_id().unwrap();
    let _registration = SYSTEM_SCHEDULER.write().prepare_to_block(tid).unwrap();
    crate::cpu::scheduler::yield_lp();
    unreachable!("A completed kernel thread was resumed")
}
