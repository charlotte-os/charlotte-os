pub mod round_robin;
use alloc::fmt::Debug;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::cpu::isa::lp::LpId;
use crate::cpu::scheduler::threads::{ThreadCount, ThreadId};
use crate::klib::observer::Observer;

pub trait LpScheduler: Debug + Send {
    fn get_lp_id(&self) -> LpId;
    fn get_tid(&self) -> Option<ThreadId>;
    fn is_ctx_switch_pending(&self) -> bool;
    /* The following two functions should use interior mutability to access an internal atomic
     * for safe lock free operaton. */
    fn set_ctx_switch_pending(&self);
    /* This should clear the pending context switch field and when appropriate create and submit
     * a new TimerEvent to the local TimerQueue so the pending context switch will get set to
     * true again when the event notifies */
    fn clear_ctx_switch_pending(&self);
    fn next(&mut self) -> Result<ThreadId, Error>;
    fn add_thread(&mut self, tid: ThreadId) -> Result<(), Error>;
    fn remove_thread(&mut self, tid: ThreadId) -> Result<(), Error>;
    fn is_idle(&self) -> bool;
    fn start(&mut self);
    fn stop(&mut self);
    fn thread_count(&self) -> ThreadCount;
}

#[derive(Debug)]
pub enum Error {
    NoRunnableThreads,
    ThreadAlreadyAssignedToLp,
    ThreadNotAssignedToThisLp,
}

#[derive(Debug)]
struct TimerEventObserver(AtomicBool);

impl Observer for TimerEventObserver {
    fn notify(self: Arc<Self>) {
        self.0.store(true, Ordering::Release);
    }
}
