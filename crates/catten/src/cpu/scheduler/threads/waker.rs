use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::cpu::isa::lp::LpId;
use crate::cpu::scheduler::system_scheduler::SYSTEM_SCHEDULER;
use crate::cpu::scheduler::threads::ThreadId;

/// One registration, rather than just a thread ID. Late timer notifications must
/// never wake a subsequent wait by the same thread (or a reused thread ID).
#[derive(Debug)]
pub struct Waker {
    pub(crate) thread_id: ThreadId,
    pub(crate) lp_id: LpId,
    notified: AtomicBool,
}

impl Waker {
    pub fn new(thread_id: ThreadId, lp_id: LpId) -> Self {
        Self {
            thread_id,
            lp_id,
            notified: AtomicBool::new(false),
        }
    }
}

impl crate::klib::observer::Observer for Waker {
    fn notify(self: Arc<Self>) {
        if !self.notified.swap(true, Ordering::AcqRel) {
            SYSTEM_SCHEDULER.write().wake_thread(&self);
        }
    }
}
