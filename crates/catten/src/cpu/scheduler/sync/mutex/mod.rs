use core::sync::atomic::{AtomicBool, Ordering};

use lock_api::{GuardNoSend, RawMutex, RawMutexTimed};

use crate::cpu::scheduler::sync::semaphore::Semaphore;
use crate::klib::time::duration::ExtDuration;
use crate::timers::{Timestamp, deadline_after};

pub type Mutex<T> = lock_api::Mutex<MutexCore, T>;

pub struct MutexCore {
    permits: Semaphore,
    locked: AtomicBool,
}

impl MutexCore {
    pub const fn new() -> Self {
        Self {
            permits: Semaphore::new(1),
            locked: AtomicBool::new(false),
        }
    }
}

impl Default for MutexCore {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl RawMutex for MutexCore {
    type GuardMarker = GuardNoSend;

    const INIT: Self = Self::new();

    fn lock(&self) {
        self.permits.wait(None);
        self.locked.store(true, Ordering::Release);
    }

    fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Acquire)
    }

    fn try_lock(&self) -> bool {
        if self.permits.try_wait() {
            self.locked.store(true, Ordering::Release);
            true
        } else {
            false
        }
    }

    unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
        self.permits.signal();
    }
}

unsafe impl RawMutexTimed for MutexCore {
    type Duration = ExtDuration;
    type Instant = Timestamp;

    fn try_lock_for(&self, duration: ExtDuration) -> bool {
        self.try_lock_until(deadline_after(duration))
    }

    fn try_lock_until(&self, deadline: Timestamp) -> bool {
        if self.permits.wait(Some(deadline)) {
            self.locked.store(true, Ordering::Release);
            true
        } else {
            false
        }
    }
}
