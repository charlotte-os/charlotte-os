//! Counting waits shared by kernel mutexes and firmware events. Registration and
//! signal use the same lock; a signal or deadline wins each waiter exactly once.
use alloc::collections::VecDeque;
use alloc::sync::{Arc, Weak};
use core::sync::atomic::{AtomicU8, Ordering};

use crate::cpu::isa::interface::timers::LpTimerIfce;
use crate::cpu::isa::lp::ops::get_int_state;
use crate::cpu::isa::timers::LpTimer;
use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::cpu::scheduler::system_scheduler::{SYSTEM_SCHEDULER, get_thread_id};
use crate::cpu::scheduler::threads::waker::Waker;
use crate::cpu::scheduler::yield_lp;
use crate::klib::observer::{Observable, Observer};
use crate::timers::{TIMER_QUEUES, TimerEvent, Timestamp};

const PENDING: u8 = 0;
const SIGNALED: u8 = 1;
const TIMED_OUT: u8 = 2;

struct Waiter {
    result: AtomicU8,
    waker: Arc<Waker>,
}

impl Observer for Waiter {
    fn notify(self: Arc<Self>) {
        if self
            .result
            .compare_exchange(PENDING, TIMED_OUT, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.waker.clone().notify();
        }
    }
}

struct State {
    count: usize,
    waiters: VecDeque<Weak<Waiter>>,
}

pub struct Semaphore {
    state: Mutex<State>,
}

impl Semaphore {
    pub const fn new(count: usize) -> Self {
        Self {
            state: Mutex::new(State {
                count,
                waiters: VecDeque::new(),
            }),
        }
    }

    pub fn try_wait(&self) -> bool {
        let mut state = self.state.lock();
        if state.count == 0 {
            false
        } else {
            state.count -= 1;
            true
        }
    }

    pub fn wait(&self, deadline: Option<Timestamp>) -> bool {
        // Boot code can use uncontended locks before the scheduler exists.
        let can_park = get_int_state();
        let thread = get_thread_id();
        if !can_park || thread.is_none() {
            loop {
                if self.try_wait() {
                    return true;
                }
                if deadline.is_some_and(|deadline| LpTimer::now() >= deadline) {
                    return false;
                }
                core::hint::spin_loop();
            }
        }
        let waiter = {
            let mut state = self.state.lock();
            if state.count > 0 {
                state.count -= 1;
                return true;
            }
            if deadline.is_some_and(|deadline| LpTimer::now() >= deadline) {
                return false;
            }
            // Timed-out waiters otherwise accumulate forever if never signaled.
            state.waiters.retain(|waiter| {
                waiter
                    .upgrade()
                    .is_some_and(|waiter| waiter.result.load(Ordering::Acquire) == PENDING)
            });
            let waker = SYSTEM_SCHEDULER
                .write()
                .prepare_to_block(thread.unwrap())
                .expect("Cannot block this execution context");
            let waiter = Arc::new(Waiter {
                result: AtomicU8::new(PENDING),
                waker,
            });
            state.waiters.push_back(Arc::downgrade(&waiter));
            if let Some(deadline) = deadline {
                let event = TimerEvent::from(deadline);
                event.register_observer(Arc::downgrade(&waiter) as Weak<dyn Observer>);
                TIMER_QUEUES.get_mut().add_event(event);
            }
            waiter
        };
        // The condition lock is gone before entering the context switch. A
        // notifier racing here makes the current thread runnable again.
        yield_lp();
        waiter.result.load(Ordering::Acquire) == SIGNALED
    }

    pub fn signal(&self) {
        let mut state = self.state.lock();
        while let Some(waiter) = state.waiters.pop_front() {
            if let Some(waiter) = waiter.upgrade() {
                if waiter
                    .result
                    .compare_exchange(PENDING, SIGNALED, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    // Transfer the newly signaled count directly to this waiter.
                    // Keeping the lock until wake prevents other waiters stealing it.
                    waiter.waker.clone().notify();
                    return;
                }
            }
        }
        state.count = state.count.saturating_add(1);
    }

    pub fn reset(&self) {
        self.state.lock().count = 0;
    }
}
