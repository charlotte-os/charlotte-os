use alloc::sync::Weak;
use core::hint::unreachable_unchecked;

use crate::cpu::scheduler::system_scheduler::SYSTEM_SCHEDULER;
use crate::cpu::scheduler::threads::{MASTER_THREAD_TABLE, Thread, ThreadId};
use crate::klib::observer::{Observable as _, Observer};
use crate::klib::time::duration::ExtDuration;
use crate::logln;
use crate::memory::AddressSpaceId;
use crate::timers::{TIMER_QUEUES, TimerEvent};

pub mod lp_schedulers;
pub mod sync;
pub mod system_scheduler;
pub mod threads;

/// Creates a new thread and submit it to the system scheduler for assignment to a logical processor
/// and then execution.
pub fn spawn_thread(asid: AddressSpaceId, entry_point: extern "C" fn()) -> ThreadId {
    let thread = Thread::new(asid, entry_point);
    let tid = MASTER_THREAD_TABLE.write().add_element(thread);
    SYSTEM_SCHEDULER
        .read()
        .submit_ready_thread(tid as ThreadId)
        .expect("Error submitting ready thread to system scheduler");
    tid
}

/// Spawn a thread that always runs on the specified logical processor.
pub fn spawn_thread_on_lp(
    asid: AddressSpaceId,
    entry_point: extern "C" fn(),
    lp_id: crate::cpu::isa::lp::LpId,
) -> ThreadId {
    let mut thread = Thread::new(asid, entry_point);
    thread.affinity = Some(lp_id);
    let tid = MASTER_THREAD_TABLE.write().add_element(thread);
    SYSTEM_SCHEDULER.read().submit_ready_thread(tid).expect("Error submitting pinned thread");
    tid
}

/// Unconditionally yields the current logical processor to the scheduler for a context switch.
///
/// This can safely be called from anywhere including outside of thread context. However if it is
/// called from interrupt context then it will cause an immediate context switch never to return
/// which will essentially cause the remainder of the ISR to get skipped. This is almost never what
/// is intended thus for interrupt service it is recommended instead to set the context switch
/// pending variable on the current LP's local scheduler and then have the switch happen at the end
/// of the ISR at which point all ISRs with the sole exception of double fault and other ISA
/// specific analogues call `cond_yield_lp` to carry out pending context switches.
pub fn yield_lp() {
    SYSTEM_SCHEDULER.read().get_lp_scheduler().lock().set_ctx_switch_pending();
    crate::cpu::isa::lp::ops::cond_yield_lp();
}

/// Aborts the current thread without calling any exit handlers.
///
/// This is the default way to exit a thread in the kernel since kernel threads should not carry any
/// state that is so complex that it requires exit handlers. For the userspace exit call this should
/// only be called after exit handlers have been run and any pending upcalls have been attempted to
/// be delivered. It is expected that exit handlers will be called from userspace itself via a given
/// program's runtime library, however upcalls are still solely the purview of the kernel and we
/// should at least attempt delivery prior to abort.
pub fn abort() -> ! {
    if let Some(tid) = SYSTEM_SCHEDULER.read().get_lp_scheduler().lock().get_tid() {
        logln!("Thread {} is aborting execution.", tid);
        SYSTEM_SCHEDULER.read().abort_thread(tid).expect("Error aborting thread");
    }
    yield_lp();
    unsafe { unreachable_unchecked() }
}

/// Blocks the current thread for at least the specified duration.
pub fn sleep(duration: ExtDuration) {
    if duration.as_picos() == 0 {
        return;
    }
    // A zero-count semaphore can only be woken by its deadline.
    sync::semaphore::Semaphore::new(0).wait(Some(crate::timers::deadline_after(duration)));
}

/// Registers an observer to be notified when the specified thread exits.
pub fn observe_thread_exit(
    thread_id: ThreadId,
    observer: Weak<dyn Observer>,
) -> Result<(), system_scheduler::Error> {
    if let Ok(thread) = MASTER_THREAD_TABLE.read().get(thread_id) {
        thread.register_observer(observer);
        Ok(())
    } else {
        Err(system_scheduler::Error::InvalidThread)
    }
}
