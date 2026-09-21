//! # Kernel Timer System

use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Weak;

use concurrent_queue::ConcurrentQueue;
use spin::LazyLock;

use crate::cpu::isa::interface::timers::{LpTimerError, LpTimerIfce};
use crate::cpu::isa::timers::LpTimer;
use crate::cpu::multiprocessor::spin::per_lp::PerLp;
use crate::klib::observer::{Observable, Observer};
use crate::klib::time::duration::ExtDuration;

pub static TIMER_QUEUES: LazyLock<PerLp<TimerQueue>> =
    LazyLock::new(|| PerLp::new(TimerQueue::default));

pub type Timestamp = <LpTimer as LpTimerIfce>::Timestamp;

/// Round upwards so a timer never expires before the requested duration.
pub fn deadline_after(duration: ExtDuration) -> Timestamp {
    let ticks = duration.as_picos().div_ceil(LpTimer::get_ts_cycle_period().as_picos());
    LpTimer::now().saturating_add(ticks.min(u64::MAX as u128) as u64)
}

/// A timer event that should notify observers when a specified deadline is reached. The deadline
/// can be set using either a duration or an absolute timestamp.
#[derive(Debug)]
pub struct TimerEvent {
    deadline: Timestamp,
    observers: ConcurrentQueue<Weak<dyn Observer>>,
}

impl TimerEvent {
    #[inline(always)]
    pub fn get_deadline(&self) -> Timestamp {
        self.deadline
    }

    fn signal(&self) {
        for observer in self.observers.try_iter() {
            if let Some(observer) = observer.upgrade() {
                observer.notify();
            }
        }
    }
}

impl From<Timestamp> for TimerEvent {
    fn from(deadline: Timestamp) -> Self {
        Self {
            deadline,
            observers: ConcurrentQueue::unbounded(),
        }
    }
}

impl From<ExtDuration> for TimerEvent {
    fn from(duration: ExtDuration) -> Self {
        let deadline = deadline_after(duration);
        Self {
            deadline,
            observers: ConcurrentQueue::unbounded(),
        }
    }
}

impl Observable for TimerEvent {
    #[inline]
    fn register_observer(&self, observer: Weak<dyn Observer>) {
        self.observers.push(observer).expect("Failed to register observer");
    }
}

#[derive(Debug, Default)]
pub struct TimerQueue {
    events: VecDeque<TimerEvent>,
}

impl TimerQueue {
    pub fn add_event(&mut self, event: TimerEvent) {
        let index = self
            .events
            .iter()
            .position(|queued| event.deadline < queued.deadline)
            .unwrap_or(self.events.len());
        self.events.insert(index, event);
        if index == 0 {
            // Registration can occur while scheduler or condition locks are held.
            // Never invoke observers synchronously here: an expired event must be
            // delivered by an interrupt after the caller releases those locks.
            let timer = LpTimer::get();
            let mut timer = timer.lock();
            match timer.set_deadline(self.events.front().unwrap().deadline) {
                Ok(()) => {}
                Err(LpTimerError::DeadlinePassed) => {
                    timer
                        .set_duration(ExtDuration::from_micros(1))
                        .expect("Cannot arm expired timer event");
                }
                Err(error) => panic!("Cannot arm timer: {:?}", error),
            }
            timer.start().expect("Failed to start timer");
        }
    }

    pub fn process_events(&mut self) {
        loop {
            let Some(event) = self.events.front() else {
                let _ = LpTimer::get().lock().stop();
                return;
            };
            if event.deadline <= LpTimer::now() {
                let event = self.events.pop_front().unwrap();
                event.signal();
                continue;
            }
            let timer = LpTimer::get();
            let mut timer = timer.lock();
            match timer.set_deadline(event.deadline) {
                Err(LpTimerError::DeadlinePassed) => continue,
                Ok(()) => {
                    timer.start().expect("Failed to start timer");
                    return;
                }
                Err(error) => panic!("Cannot arm timer: {:?}", error),
            }
        }
    }

    fn get_next_deadline(&self) -> Option<Timestamp> {
        self.events.front().map(|event| event.deadline)
    }
}
