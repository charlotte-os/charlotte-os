use core::sync::atomic::{AtomicBool, Ordering};

use lock_api::{RawMutex, RawMutexTimed};
use uacpi_wrapper::*;

use super::c_alloc::{allocate_object, free_object};
use crate::cpu::isa::lp::ops::{get_int_state, get_lp_id, mask_interrupts, unmask_interrupts};
use crate::cpu::scheduler::sync::mutex::MutexCore;
use crate::cpu::scheduler::sync::semaphore::Semaphore;
use crate::cpu::scheduler::system_scheduler::get_thread_id;
use crate::klib::time::duration::ExtDuration;
use crate::timers::deadline_after;

/// A counting event also used by the uACPI deferred-work dispatcher.
pub(super) struct Event(Semaphore);

impl Event {
    pub(super) const fn new() -> Self {
        Self(Semaphore::new(0))
    }

    pub(super) fn signal(&self) {
        self.0.signal();
    }

    pub(super) fn wait(&self, timeout: u16) -> bool {
        match timeout {
            0 => self.0.try_wait(),
            u16::MAX => self.0.wait(None),
            ms => self.0.wait(Some(deadline_after(ExtDuration::from_millis(ms as u128)))),
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_create_mutex() -> uacpi_handle {
    allocate_object(MutexCore::new()).cast()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_free_mutex(handle: uacpi_handle) {
    unsafe { free_object(handle.cast::<MutexCore>()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_acquire_mutex(
    handle: uacpi_handle,
    timeout: uacpi_u16,
) -> uacpi_status {
    let Some(mutex) = (unsafe { handle.cast::<MutexCore>().as_ref() }) else {
        return UACPI_STATUS_INVALID_ARGUMENT;
    };
    let acquired = match timeout {
        0 => mutex.try_lock(),
        u16::MAX => {
            mutex.lock();
            true
        }
        ms => mutex.try_lock_for(ExtDuration::from_millis(ms as u128)),
    };
    if acquired {
        UACPI_STATUS_OK
    } else {
        UACPI_STATUS_TIMEOUT
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_release_mutex(handle: uacpi_handle) {
    if let Some(mutex) = unsafe { handle.cast::<MutexCore>().as_ref() } {
        unsafe { mutex.unlock() };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_create_event() -> uacpi_handle {
    allocate_object(Event::new()).cast()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_free_event(handle: uacpi_handle) {
    unsafe { free_object(handle.cast::<Event>()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_wait_for_event(
    handle: uacpi_handle,
    timeout: uacpi_u16,
) -> uacpi_bool {
    unsafe { handle.cast::<Event>().as_ref() }.is_some_and(|event| event.wait(timeout))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_signal_event(handle: uacpi_handle) {
    if let Some(event) = unsafe { handle.cast::<Event>().as_ref() } {
        event.signal();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_reset_event(handle: uacpi_handle) {
    if let Some(event) = unsafe { handle.cast::<Event>().as_ref() } {
        event.0.reset();
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
