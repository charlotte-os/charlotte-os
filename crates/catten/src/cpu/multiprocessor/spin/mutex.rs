use core::sync::atomic::{AtomicBool, Ordering};

use lock_api::RawMutex;

use crate::cpu::isa::lp::ops::{get_int_state, mask_interrupts, unmask_interrupts};

pub type Mutex<T> = lock_api::Mutex<MutexCore, T>;

/// # A spinlock-based mutex that disables interrupts on the calling processor while locked.
/// This lock is suitable for providing mutual exclusion during critical sections but it should be
/// used with caution to avoid deadlocks between LPs. It prevents self deadlocks by
/// masking maskable interrupts. It exists solely for use by the global allocator and should not be
/// used for any other purpose.
#[derive(Debug)]
pub struct MutexCore {
    state: AtomicBool,
    saved_interrupt_flag: AtomicBool,
}

impl MutexCore {
    pub const fn new() -> Self {
        Self {
            state: AtomicBool::new(false),
            saved_interrupt_flag: AtomicBool::new(false),
        }
    }
}

impl Default for MutexCore {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl RawMutex for MutexCore {
    type GuardMarker = lock_api::GuardSend;

    const INIT: Self = Self::new();

    fn lock(&self) {
        let int_state = get_int_state();

        mask_interrupts!();
        while self.state.swap(true, core::sync::atomic::Ordering::Acquire) {
            core::hint::spin_loop();
        }
        self.saved_interrupt_flag.store(int_state, core::sync::atomic::Ordering::Release);
    }

    unsafe fn unlock(&self) {
        let restore = self.saved_interrupt_flag.swap(false, Ordering::Relaxed);
        self.state.store(false, Ordering::Release);
        if restore {
            unmask_interrupts!();
        }
    }

    fn try_lock(&self) -> bool {
        let int_state = get_int_state();
        mask_interrupts!();
        let locked = self.state.swap(true, core::sync::atomic::Ordering::Acquire);
        if !locked {
            self.saved_interrupt_flag.store(int_state, core::sync::atomic::Ordering::Release);
        } else {
            if int_state {
                unmask_interrupts!();
            }
        }
        !locked
    }
}
