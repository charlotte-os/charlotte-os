use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use core::sync::atomic::{AtomicBool, Ordering};

use concurrent_queue::ConcurrentQueue;
use lock_api::{GuardNoSend, RawMutex, RawMutexTimed};

use crate::cpu::scheduler::system_scheduler::{SYSTEM_SCHEDULER, get_thread_id};
use crate::cpu::scheduler::threads::{MASTER_THREAD_TABLE, ThreadId, ThreadState, waker};
use crate::klib::observer::{CallOnNotify, Observable, Observer};
use crate::klib::time::duration::ExtDuration;
use crate::klib::time::instant::ExtInstant;
use crate::timers::{TIMER_QUEUES, TimerEvent, Timestamp};

pub type Mutex<T> = lock_api::Mutex<MutexCore, T>;

#[derive(Debug)]
pub struct MutexCore {
    raw_lock: AtomicBool,
    waitlist: ConcurrentQueue<Weak<dyn Observer>>,
}

impl Default for MutexCore {
    fn default() -> Self {
        Self::new()
    }
}

impl MutexCore {
    pub fn new() -> Self {
        MutexCore {
            raw_lock: AtomicBool::new(false),
            waitlist: ConcurrentQueue::unbounded(),
        }
    }
}

impl Observable for MutexCore {
    fn register_observer(&self, observer: Weak<dyn Observer>) {
        self.waitlist.push(observer).expect("Failed to register observer");
    }
}

unsafe impl RawMutex for MutexCore {
    type GuardMarker = GuardNoSend;

    const INIT: Self = MutexCore {
        raw_lock: AtomicBool::new(false),
        waitlist: ConcurrentQueue::unbounded(),
    };

    fn lock(&self) {
        loop {
            if self
                .raw_lock
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                break; // acquired — return
            }
            // Failed to acquire — block this thread until unlock() wakes us
            if let Some(tid) = get_thread_id() {
                SYSTEM_SCHEDULER.write().block_thread(tid, self).expect("Failed to block thread");
            } else {
                panic!("Attempted to acquire a blocking mutex from outside thread context.");
            }
        }
    }

    fn is_locked(&self) -> bool {
        self.raw_lock.load(Ordering::Acquire)
    }

    fn try_lock(&self) -> bool {
        self.waitlist.is_empty()
            && self
                .raw_lock
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
    }

    unsafe fn unlock(&self) {
        self.raw_lock.store(false, Ordering::Release);
        // Wake the next waiter *after* releasing the lock
        while let Ok(observer) = self.waitlist.pop() {
            if let Some(observer) = observer.upgrade() {
                observer.notify();
                break;
            }
        }
    }
}

struct TimeoutObserver<F: Fn(ThreadId) + Send + Sync> {
    callback: F,
    thread_id: ThreadId,
}

impl<F: Fn(ThreadId) + Send + Sync> TimeoutObserver<F> {
    pub fn new(callback: F) -> Arc<Self> {
        Arc::new(TimeoutObserver {
            callback,
            thread_id: get_thread_id().expect("Failed to get thread ID"),
        })
    }
}

impl<F: Fn(ThreadId) + Send + Sync> Observer for TimeoutObserver<F> {
    fn notify(self: Arc<Self>) {
        (self.callback)(self.thread_id);
    }
}

unsafe impl RawMutexTimed for MutexCore {
    type Duration = ExtDuration;
    type Instant = Timestamp;

    fn try_lock_for(&self, timeout: Self::Duration) -> bool {
        if self.try_lock() {
            return true;
        } else {
            // Failed to acquire — block this thread until unlock() wakes us
            if let Some(tid) = get_thread_id() {
                let timer_observer = TimeoutObserver::new(|tid| {
                    if let Ok(thread) = MASTER_THREAD_TABLE.write().get_mut(tid) {
                        if let ThreadState::Blocked(waker) = &mut thread.state {
                            (*waker).clone().notify();
                        }
                    }
                });
                let timeout_event = TimerEvent::from(timeout);
                timeout_event
                    .register_observer(Arc::downgrade(&(timer_observer as Arc<dyn Observer>)));
                TIMER_QUEUES.get_mut().add_event(timeout_event);
                SYSTEM_SCHEDULER.write().block_thread(tid, self).expect("Failed to block thread");
                self.try_lock()
            } else {
                panic!("Attempted to acquire a blocking mutex from outside thread context.");
            }
        }
    }

    fn try_lock_until(&self, timeout: Self::Instant) -> bool {
        if self.try_lock() {
            return true;
        } else {
            // Failed to acquire — block this thread until unlock() wakes us
            if let Some(tid) = get_thread_id() {
                let timer_observer = TimeoutObserver::new(|tid| {
                    if let Ok(thread) = MASTER_THREAD_TABLE.write().get_mut(tid) {
                        if let ThreadState::Blocked(waker) = &mut thread.state {
                            (*waker).clone().notify();
                        }
                    }
                });
                let timeout_event = TimerEvent::from(timeout);
                timeout_event
                    .register_observer(Arc::downgrade(&(timer_observer as Arc<dyn Observer>)));
                TIMER_QUEUES.get_mut().add_event(timeout_event);
                SYSTEM_SCHEDULER.write().block_thread(tid, self).expect("Failed to block thread");
                self.try_lock()
            } else {
                panic!("Attempted to acquire a blocking mutex from outside thread context.");
            }
        }
    }
}
