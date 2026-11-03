use core::sync::atomic::{AtomicBool, Ordering};

use uacpi_wrapper::*;

use super::c_alloc::{allocate_object, free_object};
use crate::cpu::isa::lp::ops::{get_int_state, get_lp_id, mask_interrupts, unmask_interrupts};
use crate::cpu::scheduler::sync::semaphore::Semaphore;
use crate::cpu::scheduler::system_scheduler::get_thread_id;
use crate::klib::time::duration::ExtDuration;
use crate::timers::deadline_after;

/// Takes one unit from `semaphore`, interpreting `timeout` the way uACPI does:
/// 0 is a single attempt, 0xFFFF waits forever, and anything else is a bound in
/// milliseconds.
fn take_unit(semaphore: &Semaphore, timeout: u16) -> bool {
    match timeout {
        0 => semaphore.try_wait(),
        u16::MAX => semaphore.wait(None),
        ms => semaphore.wait(Some(deadline_after(ExtDuration::from_millis(ms as u128)))),
    }
}

// uACPI uses one semaphore type both as its locks (one unit) and its events
// (zero units, unbounded). The scheduler semaphore has no owner and its signal
// never sleeps, which both uses require.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_create_semaphore(initial_units: uacpi_u32) -> uacpi_handle {
    allocate_object(Semaphore::new(initial_units as usize)).cast()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_free_semaphore(handle: uacpi_handle) {
    unsafe { free_object(handle.cast::<Semaphore>()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_wait_for_semaphore(
    handle: uacpi_handle,
    timeout: uacpi_u16,
) -> uacpi_status {
    let Some(semaphore) = (unsafe { handle.cast::<Semaphore>().as_ref() }) else {
        return UACPI_STATUS_INVALID_ARGUMENT;
    };
    if take_unit(semaphore, timeout) {
        UACPI_STATUS_OK
    } else {
        UACPI_STATUS_TIMEOUT
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_signal_semaphore(handle: uacpi_handle) {
    if let Some(semaphore) = unsafe { handle.cast::<Semaphore>().as_ref() } {
        semaphore.signal();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_get_thread_id() -> uacpi_thread_id {
    // Low IDs name scheduler threads, high IDs name boot/idle contexts per LP.
    // Neither range includes uACPI's all-ones NONE sentinel.
    get_thread_id().unwrap_or((1usize << (usize::BITS - 1)) | get_lp_id() as usize)
        as uacpi_thread_id
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_disable_interrupts() -> uacpi_interrupt_state {
    let enabled = get_int_state();
    mask_interrupts!();
    enabled as uacpi_interrupt_state
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_restore_interrupts(state: uacpi_interrupt_state) {
    if state != 0 {
        unmask_interrupts!();
    } else {
        mask_interrupts!();
    }
}

struct Spinlock(AtomicBool);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_create_spinlock() -> uacpi_handle {
    allocate_object(Spinlock(AtomicBool::new(false))).cast()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_free_spinlock(handle: uacpi_handle) {
    unsafe { free_object(handle.cast::<Spinlock>()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_lock_spinlock(handle: uacpi_handle) -> uacpi_cpu_flags {
    let flags = get_int_state() as uacpi_cpu_flags;
    mask_interrupts!();
    let lock = unsafe { &*handle.cast::<Spinlock>() };
    while lock.0.swap(true, Ordering::Acquire) {
        core::hint::spin_loop();
    }
    flags
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_unlock_spinlock(
    handle: uacpi_handle,
    flags: uacpi_cpu_flags,
) {
    unsafe { &*handle.cast::<Spinlock>() }.0.store(false, Ordering::Release);
    if flags != 0 {
        unmask_interrupts!();
    }
}
