//! # Low-level operations for x86_64 Logical Processors

pub fn init_lp_state() {
    unsafe {
        core::arch::asm! {
            "mov rax, cr4",
            "or rax, 1<<16",
            "mov cr4, rax",
            "mov rax, 0",
            "wrfsbase rax",
            "wrgsbase rax",
            out("rax") _
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

use core::arch::{
    asm,
    naked_asm,
};

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

use crate::{
    cpu::scheduler::{
        system_scheduler::SYSTEM_SCHEDULER,
        threads::MASTER_THREAD_TABLE,
    },
    logln,
    memory::VirtualAddress,
};

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
    #[cfg(feature = "yield_trace")]
    #[derive(Clone, Copy)]
    enum YieldTrace {
        None,
        NoSwitch {
            lp_id: LpId,
        },
        FromThread {
            current: usize,
            next: usize,
            lp_id: LpId,
        },
        FromNonThread {
            next: usize,
            lp_id: LpId,
        },
    }
    #[cfg(feature = "yield_trace")]
    let mut trace = YieldTrace::None;
    // Collect switch parameters and release all locks before calling switch_ctx.
    // switch_ctx may permanently abandon the current stack (initial non-thread switch),
    // so any guards held across it would never be dropped, leaving locks permanently locked.
    let switch_params: Option<(*mut VirtualAddress, *const VirtualAddress)> = {
        let sched = SYSTEM_SCHEDULER.read();
        let mut lsched = sched.get_lp_scheduler().lock();
        if lsched.is_ctx_switch_pending() {
            let curr_tid = lsched.get_tid();
            if let Ok(next_tid) = lsched.next() {
                if curr_tid.is_some() {
                    if next_tid != curr_tid.unwrap() {
                        let (curr_rsp0_ptr, next_rsp0_ptr) = {
                            let mut tt_guard = MASTER_THREAD_TABLE.write();
                            let curr_thread = tt_guard
                                .get_mut(
                                    curr_tid.expect("Current thread ID not found during yield."),
                                )
                                .expect("Current thread not found during yield.");
                            let curr_rsp0_ptr =
                                &raw mut curr_thread.context.kernel_stack_buf.curr_sp;
                            let next_thread = tt_guard
                                .get_mut(next_tid)
                                .expect("Next thread not found during yield.");
                            let next_rsp0_ptr =
                                &raw mut next_thread.context.kernel_stack_buf.curr_sp;
                            (curr_rsp0_ptr, next_rsp0_ptr)
                        };
                        cfg_select! {
                            feature = "yield_trace" => {
                                trace = YieldTrace::FromThread {
                                    current: curr_tid.unwrap(),
                                    next: next_tid,
                                    lp_id: get_lp_id(),
                                };
                            }
                            _ => {}
                        }
                        lsched.clear_ctx_switch_pending();
                        Some((curr_rsp0_ptr, next_rsp0_ptr))
                    } else {
                        cfg_select! {
                            feature = "yield_trace" => {
                                trace = YieldTrace::NoSwitch {
                                    lp_id: get_lp_id(),
                                };
                            }
                            _ => {}
                        }
                        None
                    }
                } else {
                    let next_rsp0_ptr = {
                        let mut tt_guard = MASTER_THREAD_TABLE.write();
                        let next_thread = tt_guard
                            .get_mut(next_tid)
                            .expect("Next thread not found during yield.");
                        &raw mut next_thread.context.kernel_stack_buf.curr_sp
                    };
                    cfg_select! {
                        feature = "yield_trace" => {
                            trace = YieldTrace::FromNonThread {
                                next: next_tid,
                                lp_id: get_lp_id(),
                            };
                        }
                        _ => {}
                    }
                    lsched.clear_ctx_switch_pending();
                    Some((core::ptr::null_mut(), next_rsp0_ptr))
                }
            } else {
                logln!(
                    "LP {:?}: No runnable threads found during yield, even though a context \
                     switch was pending. Awaiting interrupt...",
                    (get_lp_id())
                );
                await_interrupt!();
            }
        } else {
            None
        }
        // lsched and sched guards dropped here before switch_ctx
    };
    #[cfg(feature = "yield_trace")]
    match trace {
        YieldTrace::None => {}
        YieldTrace::NoSwitch {
            lp_id,
        } => logln!(
            "No thread switch needed during yield on LP {:?} because the next thread is the same \
             as the current thread.",
            lp_id
        ),
        YieldTrace::FromThread {
            current,
            next,
            lp_id,
        } => logln!("Yielding from thread {:?} to thread {:?} on LP {:?}", current, next, lp_id),
        YieldTrace::FromNonThread {
            next,
            lp_id,
        } => logln!("Yielding from non-thread context to thread {:?} on LP {:?}", next, lp_id),
    }
    if let Some((curr_rsp0_ptr, next_rsp0_ptr)) = switch_params {
        switch_ctx(curr_rsp0_ptr as *mut u64, next_rsp0_ptr as *const u64);
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
        "add rsp, 8",
        "cli",
        "2:",
        "hlt",
        "jmp 2b",
    );
}
