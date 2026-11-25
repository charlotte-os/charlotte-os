use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU32};

use crate::cpu::isa::lp::LpId;
use crate::timers::TimerQueue;

pub struct LogicalProcessorData {
    pub id: LpId,
    pub interrupt_depth: AtomicU16,
    pub interrupt_save_count: AtomicU32,
    pub saved_interrupt_flag: AtomicBool,
    pub timer_queue: TimerQueue,
}
