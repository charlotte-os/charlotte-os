use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;

use super::lp_schedulers::LpScheduler;
use crate::cpu::isa::constants::interrupt_vectors::LAPIC_TIMER_VECTOR;
use crate::cpu::isa::interface::interrupts::LocalIntCtlrIfce;
use crate::cpu::isa::interrupts::LocalIntCtlr;
use crate::cpu::isa::lp::LpId;
use crate::cpu::isa::lp::ops::get_lp_id;
use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::cpu::multiprocessor::spin::rwlock::RwLock;
use crate::cpu::scheduler::threads::{MASTER_THREAD_TABLE, ThreadId, ThreadState, waker};
use crate::logln;
use crate::memory::AddressSpaceId;

pub static SYSTEM_SCHEDULER: RwLock<SystemScheduler> = RwLock::new(SystemScheduler::new());

#[derive(Debug)]
pub enum Error {
    InvalidThread,
    AlreadyBlocked,
    ThreadTerminated,
}

/// The system-wide thread scheduler
pub struct SystemScheduler {
    lp_schedulers: BTreeMap<LpId, Mutex<Box<dyn LpScheduler>>>,
}

impl SystemScheduler {
    pub const fn new() -> Self {
        Self {
            lp_schedulers: BTreeMap::new(),
        }
    }

    pub unsafe fn set_lp_scheduler(&mut self, lp_sched: Box<dyn LpScheduler>) {
        //! Safety: This function should only be called once per LP at boot during the BSP and AP
        //! init processes and it must be called in the same order that LP IDs were assigned
        //! otherwise the wrong LP will use the wrong local scheduler.
        let ls_sync_ptr = Mutex::new(lp_sched);
        self.lp_schedulers.insert(get_lp_id(), ls_sync_ptr);
    }

    pub fn get_lp_scheduler(&self) -> &Mutex<Box<dyn LpScheduler>> {
        &self.lp_schedulers[&get_lp_id()]
    }

    pub fn submit_ready_thread(&self, tid: ThreadId) -> Result<LpId, Error> {
        let affinity =
            MASTER_THREAD_TABLE.read().get(tid).map_err(|_| Error::InvalidThread)?.affinity;
        let target = match affinity {
            Some(lp) => self.lp_schedulers.get(&lp).ok_or(Error::InvalidThread)?,
            None => self.get_least_loaded_lp(),
        };
        let mut scheduler = target.lock();
        let was_idle = scheduler.is_idle();
        scheduler.add_thread(tid).map_err(|_| Error::InvalidThread)?;
        let lp_id = scheduler.get_lp_id();
        scheduler.set_ctx_switch_pending();
        drop(scheduler);
        if was_idle && lp_id != get_lp_id() {
            let _ = LocalIntCtlr::send_unicast_ipi(lp_id, LAPIC_TIMER_VECTOR);
        }
        Ok(lp_id)
    }

    /// Mark the current thread blocked, retaining its current handle until the
    /// context switch saves its stack. The caller must serialize registration
    /// with its condition, release all locks, and then call `yield_lp`.
    pub fn prepare_to_block(&self, tid: ThreadId) -> Result<Arc<waker::Waker>, Error> {
        let lp_id = get_lp_id();
        let scheduler = self.get_lp_scheduler().lock();
        if scheduler.get_tid() != Some(tid) {
            return Err(Error::InvalidThread);
        }
        let mut threads = MASTER_THREAD_TABLE.write();
        let thread = threads.get_mut(tid).map_err(|_| Error::InvalidThread)?;
        if !matches!(thread.state, ThreadState::Running(_)) {
            return Err(Error::AlreadyBlocked);
        }
        let waker = Arc::new(waker::Waker::new(tid, lp_id));
        thread.state = ThreadState::Blocked(waker.clone());
        scheduler.set_ctx_switch_pending();
        Ok(waker)
    }

    /// Wake only this registration, and resume on the processor that saved the
    /// context. A remote processor must not run a stack that is still executing.
    pub fn wake_thread(&self, wake: &Arc<waker::Waker>) {
        let mut scheduler = self.lp_schedulers[&wake.lp_id].lock();
        let mut threads = MASTER_THREAD_TABLE.write();
        let Ok(thread) = threads.get_mut(wake.thread_id) else {
            return;
        };
        if !matches!(&thread.state, ThreadState::Blocked(active) if Arc::ptr_eq(active, wake)) {
            return;
        }
        if scheduler.get_tid() == Some(wake.thread_id) {
            // Notification raced with parking; this context is still current.
            thread.state = ThreadState::Running(wake.lp_id);
        } else {
            thread.state = ThreadState::NeedsLpAssignment;
            drop(threads);
            scheduler.add_thread(wake.thread_id).expect("Failed to resume blocked thread");
        }
        scheduler.set_ctx_switch_pending();
        let was_idle = scheduler.is_idle();
        drop(scheduler);
        if was_idle && wake.lp_id != get_lp_id() {
            let _ = LocalIntCtlr::send_unicast_ipi(wake.lp_id, LAPIC_TIMER_VECTOR);
        }
    }

    /// Register a wait while the event's condition is locked. Callers must yield
    /// after dropping their condition and scheduler guards.
    pub fn block_thread(
        &mut self,
        tid: ThreadId,
        event: &dyn crate::klib::observer::Observable,
    ) -> Result<(), Error> {
        let waker = self.prepare_to_block(tid)?;
        event
            .register_observer(Arc::downgrade(&waker) as Weak<dyn crate::klib::observer::Observer>);
        Ok(())
    }

    pub fn abort_thread(&self, tid: ThreadId) -> Result<ThreadId, Error> {
        if let Ok(thread) = MASTER_THREAD_TABLE.write().get_mut(tid) {
            match thread.state {
                ThreadState::Running(lp_id) | ThreadState::Ready(lp_id) => {
                    self.lp_schedulers[&lp_id]
                        .lock()
                        .remove_thread(tid)
                        .expect("Error removing thread from LP scheduler while aborting");
                }
                _ => {}
            }
            MASTER_THREAD_TABLE
                .write()
                .remove_element(tid)
                .expect(&format!("Failed to delete thread {tid}"));
            Ok(tid)
        } else {
            Err(Error::InvalidThread)
        }
    }

    pub fn abort_as_threads(&self, asid: AddressSpaceId) {
        let mut threads_to_abort = Vec::new();
        for (id, thread) in MASTER_THREAD_TABLE.read().iter().enumerate() {
            if let Some(thread) = thread {
                if thread.asid == asid {
                    threads_to_abort.push(id);
                }
            }
        }
        for tid in threads_to_abort {
            self.abort_thread(tid).expect("Error aborting thread by ASID");
        }
    }

    fn get_least_loaded_lp(&self) -> &Mutex<Box<dyn LpScheduler>> {
        self.lp_schedulers.iter().min_by_key(|sched| sched.1.lock().thread_count()).unwrap().1
    }
}

pub fn get_thread_id() -> Option<ThreadId> {
    let scheduler = SYSTEM_SCHEDULER.read();
    scheduler.lp_schedulers.get(&get_lp_id()).and_then(|local| local.lock().get_tid())
}
