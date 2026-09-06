use alloc::sync::Weak;
use core::ops::Deref;
use core::sync::atomic::{AtomicI64, Ordering};

use concurrent_queue::ConcurrentQueue;
use lock_api::RawRwLock;

use crate::cpu::scheduler::system_scheduler::{SYSTEM_SCHEDULER, get_thread_id};
use crate::klib::observer::{Observable, Observer};

pub type RwLock<T> = lock_api::RwLock<RwLockCore, T>;

struct Waitlist(ConcurrentQueue<Weak<dyn Observer>>);
impl Default for Waitlist {
    fn default() -> Self {
        Waitlist(ConcurrentQueue::unbounded())
    }
}

impl Observable for Waitlist {
    fn register_observer(&self, observer: Weak<dyn Observer>) {
        self.0.push(observer).expect("Failed to register observer");
    }
}

impl Deref for Waitlist {
    type Target = ConcurrentQueue<Weak<dyn Observer>>;

    fn deref(&self) -> &ConcurrentQueue<Weak<dyn Observer>> {
        &self.0
    }
}

#[derive(Default)]
pub struct RwLockCore {
    raw_lock: AtomicI64,
    waitlist_shared: Waitlist,
    waitlist_exclusive: Waitlist,
}

impl RwLockCore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RwLockCore {
    fn wake_next(&self) {
        if self.waitlist_exclusive.len() > 0 {
            if let Ok(observer) = self.waitlist_exclusive.pop() {
                if let Some(observer) = observer.upgrade() {
                    observer.notify();
                }
            }
        } else {
            while self.waitlist_shared.len() > 0 {
                while let Ok(observer) = self.waitlist_shared.pop() {
                    if let Some(observer) = observer.upgrade() {
                        observer.notify();
                    }
                }
            }
        }
    }
}

unsafe impl RawRwLock for RwLockCore {
    type GuardMarker = lock_api::GuardNoSend;

    const INIT: Self = Self {
        raw_lock: AtomicI64::new(0),
        waitlist_shared: Waitlist(ConcurrentQueue::unbounded()),
        waitlist_exclusive: Waitlist(ConcurrentQueue::unbounded()),
    };

    fn lock_exclusive(&self) {
        while self.raw_lock.compare_exchange(0, -1, Ordering::AcqRel, Ordering::Acquire).is_err() {
            SYSTEM_SCHEDULER.write().block_thread(
                get_thread_id()
                    .expect("Attempted to lock a blocking lock from outside thread context!"),
                &self.waitlist_exclusive,
            );
        }
    }

    fn try_lock_exclusive(&self) -> bool {
        if self.raw_lock.compare_exchange(0, -1, Ordering::AcqRel, Ordering::Acquire).is_err() {
            false
        } else {
            true
        }
    }

    unsafe fn unlock_exclusive(&self) {
        if self.raw_lock.compare_exchange(-1, 0, Ordering::AcqRel, Ordering::Acquire).is_ok() {
            self.wake_next();
        } else {
            panic!("Attempted to unlock an exclusive lock that was not held!");
        }
    }

    fn lock_shared(&self) {
        while self
            .raw_lock
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
                if x >= 0 {
                    Some(x + 1)
                } else {
                    None
                }
            })
            .is_err()
        {
            SYSTEM_SCHEDULER.write().block_thread(
                get_thread_id()
                    .expect("Attempted to lock a blocking lock from outside thread context!"),
                &self.waitlist_shared,
            );
        }
    }

    fn try_lock_shared(&self) -> bool {
        self.raw_lock
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
                if x >= 0 {
                    Some(x + 1)
                } else {
                    None
                }
            })
            .is_ok()
    }

    unsafe fn unlock_shared(&self) {
        if self
            .raw_lock
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
                if x > 0 {
                    Some(x - 1)
                } else {
                    None
                }
            })
            .is_ok()
        {
            self.wake_next();
        } else {
            panic!("Attempted to unlock a shared lock that was not held!");
        }
    }
}
