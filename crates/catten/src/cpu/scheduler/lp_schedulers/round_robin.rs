use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

use hashbrown::HashMap;

use crate::cpu::isa::lp::LpId;
use crate::cpu::isa::memory::paging::HwAsid;
use crate::cpu::scheduler::lp_schedulers::{Error, LpScheduler};
use crate::cpu::scheduler::threads::{MASTER_THREAD_TABLE, ThreadCount, ThreadId, ThreadState};
use crate::klib::observer::{Observable, Observer};
use crate::klib::time::duration::ExtDuration;
use crate::memory::AddressSpaceId;
use crate::timers::{TIMER_QUEUES, TimerEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ThreadHandle(ThreadId);
impl PartialOrd for ThreadHandle {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let tt_guard = MASTER_THREAD_TABLE.read();
        let self_as = unsafe { tt_guard.get(self.0).as_ref().unwrap_unchecked().asid };
        let other_as = unsafe { tt_guard.get(other.0).as_ref().unwrap_unchecked().asid };
        // Sort first by AddressSpaceId then by ThreadId
        if self_as != other_as {
            self_as.partial_cmp(&other_as)
        } else {
            self.0.partial_cmp(&other.0)
        }
    }
}
impl Ord for ThreadHandle {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        unsafe { self.partial_cmp(other).unwrap_unchecked() }
    }
}

#[derive(Debug)]
pub struct RoundRobin {
    lp_id: LpId,
    quantum: ExtDuration,
    is_idle: bool,
    timer_event_observer: Arc<super::TimerEventObserver>,
    run_queue: VecDeque<ThreadHandle>,
    current_handle: Option<ThreadHandle>,
    hwasid_map: HashMap<AddressSpaceId, HwAsid>,
}

impl RoundRobin {
    pub fn new(lp_id: LpId, quantum: ExtDuration) -> Self {
        Self {
            lp_id,
            quantum,
            is_idle: false,
            timer_event_observer: Arc::new(super::TimerEventObserver(AtomicBool::default())),
            run_queue: VecDeque::new(),
            current_handle: None,
            hwasid_map: HashMap::new(),
        }
    }

    fn set_next_timer_event(&self) {
        let mut timer_event = TimerEvent::from(self.quantum);
        timer_event.register_observer(
            Arc::downgrade(&self.timer_event_observer) as alloc::sync::Weak<dyn Observer>
        );
        TIMER_QUEUES.try_get_mut().unwrap().add_event(timer_event);
    }
}

impl LpScheduler for RoundRobin {
    fn get_lp_id(&self) -> crate::cpu::isa::lp::LpId {
        self.lp_id
    }

    fn get_tid(&self) -> Option<ThreadId> {
        if let Some(th) = self.current_handle {
            Some(th.0)
        } else {
            None
        }
    }

    fn is_ctx_switch_pending(&self) -> bool {
        self.timer_event_observer.0.load(Ordering::Acquire)
    }

    fn set_ctx_switch_pending(&self) {
        self.timer_event_observer.0.store(true, core::sync::atomic::Ordering::Release);
    }

    fn clear_ctx_switch_pending(&self) {
        self.timer_event_observer.0.store(false, core::sync::atomic::Ordering::Release);
        self.set_next_timer_event();
    }

    fn next(&mut self) -> Result<ThreadId, Error> {
        let mut threads = MASTER_THREAD_TABLE.write();
        let previous = self.current_handle;
        let previous_is_runnable = previous.is_some_and(|handle| {
            threads
                .get(handle.0)
                .is_ok_and(|thread| matches!(thread.state, ThreadState::Running(_)))
        });
        if self.run_queue.is_empty() {
            return if previous_is_runnable {
                Ok(previous.unwrap().0)
            } else {
                Err(Error::NoRunnableThreads)
            };
        }
        if previous_is_runnable {
            let previous = previous.unwrap();
            threads.get_mut(previous.0).unwrap().state = ThreadState::Ready(self.lp_id);
            self.run_queue.push_back(previous);
        }
        let next = self.run_queue.pop_front().unwrap();
        self.current_handle = Some(next);
        threads.get_mut(next.0).unwrap().state = ThreadState::Running(self.lp_id);
        self.is_idle = false;
        Ok(next.0)
    }

    fn add_thread(&mut self, tid: ThreadId) -> Result<(), Error> {
        let thread_already_assigned = {
            let tt_guard = MASTER_THREAD_TABLE.read();
            matches!(
                tt_guard.get(tid).as_ref().unwrap().state,
                ThreadState::Running(_) | ThreadState::Ready(_)
            )
        };

        if thread_already_assigned {
            Err(Error::ThreadAlreadyAssignedToLp)
        } else {
            let new_handle = ThreadHandle(tid);
            self.run_queue.push_back(new_handle);
            MASTER_THREAD_TABLE.write().get_mut(tid).as_mut().unwrap().state =
                ThreadState::Ready(self.lp_id);
            Ok(())
        }
    }

    fn remove_thread(&mut self, tid: ThreadId) -> Result<(), Error> {
        let handle = ThreadHandle(tid);

        if self.current_handle == Some(handle) {
            self.current_handle = None;
            Ok(())
        } else {
            match self.run_queue.iter().position(|queued| *queued == handle) {
                Some(idx) => {
                    self.run_queue.remove(idx);
                    Ok(())
                }
                None => Err(Error::ThreadNotAssignedToThisLp),
            }
        }
    }

    fn is_idle(&self) -> bool {
        self.is_idle
    }

    fn start(&mut self) {
        self.is_idle = false;
        self.set_next_timer_event();
    }

    fn stop(&mut self) {
        self.is_idle = true;
    }

    fn asid_to_hwasid(&self, asid: AddressSpaceId) -> Option<HwAsid> {
        self.hwasid_map.get(&asid).cloned()
    }

    fn thread_count(&self) -> ThreadCount {
        self.run_queue.len()
            + if self.current_handle.is_some() {
                1
            } else {
                0
            }
    }
}
