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
        let deadline = LpTimer::now()
            + (duration.as_picos() / LpTimer::get_ts_cycle_period().as_picos()) as Timestamp;
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
        let mut insertion_idx: Option<usize> = None;
        for (i, event_node) in self.events.iter().enumerate() {
            if event.deadline < event_node.get_deadline() {
                insertion_idx = Some(i);
                break;
            }
        }
        if insertion_idx.is_none() {
            // If we get here then the event we are adding has a deadline that is after all of the
            // other events in the queue so we can just add it to the back of the queue.
            insertion_idx = Some(self.events.len());
        }
        let i = insertion_idx.unwrap();
        self.events.insert(i, event);
        if i == 0 {
            // If the event we added is at the front of the queue then we need to prime the
            // timer with its deadline so that it will fire at the
            // correct time. If there are other events then the timer is
            // already primed with the correct deadline and will be
            // updated when the current event expires.
            if let Some(next_event) = self.events.front() {
                let timer = LpTimer::get();
                let mut timerlk = timer.lock();
                let _ = timerlk.stop();
                timerlk
                    .set_deadline(next_event.deadline)
                    .expect("Failed to set timer deadline for new event");
                timerlk.start().expect("Failed to start timer for new event");
            }
        }
    }

    pub fn process_events(&mut self) {
        while let Some(event) = self.events.front() {
            if event.get_deadline() <= LpTimer::now() {
                event.signal();
                self.events.pop_front();
            } else if let Some(deadline) = self.get_next_deadline() {
                let timer = LpTimer::get();
                let mut timerlk = timer.lock();
                if timerlk.set_deadline(deadline) == Err(LpTimerError::DeadlinePassed) {
                    continue;
                }
                timerlk.start().expect("Failed to start timer for next event");
                return;
            } else {
                let _ = LpTimer::get().lock().stop();
                return;
            }
        }
    }

    fn get_next_deadline(&self) -> Option<Timestamp> {
        self.events.front().map(|event| event.deadline)
    }
}
