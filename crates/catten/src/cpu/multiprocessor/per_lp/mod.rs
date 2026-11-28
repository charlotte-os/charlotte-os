use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU32};

use crate::cpu::isa::lp::LpId;
use crate::cpu::multiprocessor::int_state::InterruptState;
use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::cpu::scheduler::lp_schedulers::LpScheduler;
use crate::timers::TimerQueue;

pub struct LogicalProcessorData {
    pub id: LpId,
    pub interrupt_depth: AtomicU16,
    pub interrupt_state: InterruptState,
    pub timer_queue: TimerQueue,
    pub local_sched: Mutex<Box<dyn LpScheduler>>,
}

impl LogicalProcessorData {
    pub fn new(id: LpId, sched: Box<dyn LpScheduler>) -> LogicalProcessorData {
        LogicalProcessorData {
            id,
            interrupt_depth: AtomicU16::new(0),
            interrupt_state: InterruptState::default(),
            timer_queue: TimerQueue::default(),
            local_sched: Mutex::new(sched),
        }
    }
}
