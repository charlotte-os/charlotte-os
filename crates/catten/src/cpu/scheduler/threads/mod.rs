pub mod waker;

use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::mem::offset_of;

use spin::LazyLock;

use crate::cpu::isa::lp::LpId;
use crate::cpu::isa::lp::thread_context::ThreadContext;
use crate::cpu::multiprocessor::spin::rwlock::RwLock;
use crate::cpu::scheduler::threads::waker::Waker;
use crate::klib::collections::id_table::IdTable;
use crate::klib::observer::{Observable, Observer};
use crate::memory::{AddressSpaceId, KERNEL_ASID};

pub static MASTER_THREAD_TABLE: LazyLock<RwLock<ThreadTable>> =
    LazyLock::new(|| RwLock::new(ThreadTable::new()));
pub type ThreadTable = IdTable<Thread>;
pub type ThreadId = usize;

pub type ThreadCount = usize;

#[derive(Debug)]
pub enum ThreadState {
    Running(LpId),
    Ready(LpId),
    NeedsLpAssignment,
    Blocked(Arc<Waker>),
}

#[derive(Debug)]
pub struct Thread {
    pub context: ThreadContext,
    pub asid: AddressSpaceId,
    pub state: ThreadState,
    exit_observers: spin::Mutex<Vec<Weak<dyn Observer>>>,
}

pub const THREAD_CTX_OFFSET: usize = offset_of!(Thread, context);

impl Thread {
    pub fn new(asid: AddressSpaceId, entry_point: extern "C" fn()) -> Self {
        Thread {
            context: if asid != KERNEL_ASID {
                ThreadContext::create_user_thread_context(asid, entry_point)
                    .expect("Error creating user thread context")
            } else {
                ThreadContext::create_kernel_thread_context(entry_point)
                    .expect("Error creating kernel thread context")
            },
            asid,
            state: ThreadState::NeedsLpAssignment,
            exit_observers: spin::Mutex::new(Vec::new()),
        }
    }

    pub fn is_user_thread(&self) -> bool {
        self.asid != KERNEL_ASID
    }
}

/// The `Observable` trait is implemented for `Thread` to notify observers when a thread exits and
/// is dropped. This can be used to implement thread joining like functionality but also crucially
/// for monitoring when work started from a system call, nearly all of which are asynchronous in
/// Catten, finishes executing so userspace can be notified if requested. In any case the
/// completion capability returned from the system call would be registered as an observer of the
/// thread that is executing the work whose completion it represents so that userspace software can
/// monitor it in real time using the same mechanism the kernel would use.
impl Observable for Thread {
    fn register_observer(&self, observer: Weak<dyn Observer>) {
        self.exit_observers.lock().push(observer);
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        for observer in self.exit_observers.lock().iter() {
            if let Some(observer) = observer.upgrade() {
                observer.notify();
            }
        }
    }
}
