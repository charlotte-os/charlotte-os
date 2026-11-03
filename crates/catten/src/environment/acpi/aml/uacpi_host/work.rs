//! Allocation-free interrupt-side submission to persistent kernel worker threads.
//!
//! uACPI creates a work item for every piece of deferred work ahead of time, so the
//! queues are intrusive lists threaded through those items: scheduling never
//! allocates and cannot fail, which uACPI requires since it returns nothing.

use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use uacpi_wrapper::*;

use super::c_alloc::{allocate_object, free_object};
use crate::cpu::multiprocessor::interrupt_tracking::get_interrupt_depth;
use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::cpu::scheduler::sync::semaphore::Semaphore;
use crate::cpu::scheduler::{spawn_thread_on_lp, yield_lp};
use crate::memory::KERNEL_ASID;

#[derive(Clone, Copy)]
struct Work {
    handler: unsafe extern "C" fn(uacpi_handle),
    context: uacpi_handle,
}

struct WorkItem {
    next: *mut WorkItem,
    work: Option<Work>,
}

/// FIFO of pending work items, linked through `WorkItem::next`.
struct Queue {
    head: *mut WorkItem,
    tail: *mut WorkItem,
}

// The items are only ever reached through the queue's lock while pending.
unsafe impl Send for Queue {}

impl Queue {
    const fn new() -> Self {
        Self {
            head: null_mut(),
            tail: null_mut(),
        }
    }

    /// `item` must be live and not already pending.
    unsafe fn push(&mut self, item: *mut WorkItem, work: Work) {
        unsafe {
            (*item).next = null_mut();
            (*item).work = Some(work);
            match self.tail.as_mut() {
                Some(tail) => tail.next = item,
                None => self.head = item,
            }
        }
        self.tail = item;
    }

    /// Detaches the oldest item and takes its work out of it. uACPI may reuse
    /// or free the item as soon as the handler runs, so it is never touched
    /// again after this returns.
    fn pop(&mut self) -> Option<Work> {
        let item = unsafe { self.head.as_mut() }?;
        self.head = item.next;
        if self.head.is_null() {
            self.tail = null_mut();
        }
        item.next = null_mut();
        item.work.take()
    }
}

struct Worker {
    queue: Mutex<Queue>,
    available: Semaphore,
}

impl Worker {
    const fn new() -> Self {
        Self {
            queue: Mutex::new(Queue::new()),
            available: Semaphore::new(0),
        }
    }

    fn run(&self) -> ! {
        loop {
            self.available.wait(None);
            let work = self.queue.lock().pop();
            if let Some(work) = work {
                unsafe { (work.handler)(work.context) };
                OUTSTANDING.fetch_sub(1, Ordering::Release);
            }
        }
    }
}

static GPE_WORKER: Worker = Worker::new();
static NOTIFY_WORKER: Worker = Worker::new();
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static OUTSTANDING: AtomicUsize = AtomicUsize::new(0);

extern "C" fn execute_gpes() {
    GPE_WORKER.run()
}
extern "C" fn execute_notifications() {
    NOTIFY_WORKER.run()
}

/// Called once from ACPI initialization, after the BSP scheduler exists and before enabling SCI.
pub(super) fn initialize() {
    assert!(!INITIALIZED.load(Ordering::Acquire), "ACPI workers already initialized");
    spawn_thread_on_lp(KERNEL_ASID, execute_gpes, 0);
    // Notifications use a separate worker so a blocked AML GPE does not prevent notifications.
    // Keeping both on the BSP also avoids firmware that assumes all AML executes on CPU 0.
    spawn_thread_on_lp(KERNEL_ASID, execute_notifications, 0);
    INITIALIZED.store(true, Ordering::Release);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_create_work_item() -> uacpi_handle {
    allocate_object(WorkItem {
        next: null_mut(),
        work: None,
    })
    .cast()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_free_work_item(handle: uacpi_handle) {
    unsafe { free_object(handle.cast::<WorkItem>()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_schedule_work(
    work_type: uacpi_work_type,
    work_item: uacpi_handle,
    handler: uacpi_work_handler,
    context: uacpi_handle,
) {
    let item = work_item.cast::<WorkItem>();
    // uACPI never passes either as null, and there is no status to report it with.
    let Some(handler) = handler else {
        return;
    };
    if item.is_null() {
        return;
    }
    let worker = match work_type {
        UACPI_WORK_GPE_EXECUTION => &GPE_WORKER,
        _ => &NOTIFY_WORKER,
    };
    {
        let mut queue = worker.queue.lock();
        unsafe {
            queue.push(
                item,
                Work {
                    handler,
                    context,
                },
            )
        };
        OUTSTANDING.fetch_add(1, Ordering::Release);
    }
    worker.available.signal();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_wait_for_work_completion() -> uacpi_status {
    if get_interrupt_depth() != 0 {
        return UACPI_STATUS_INVALID_ARGUMENT;
    }
    super::interrupts::wait_for_interrupts();
    // Work scheduled by a handler is counted before that handler's own work is
    // retired, so this also waits for everything the drained work schedules.
    while OUTSTANDING.load(Ordering::Acquire) != 0 {
        yield_lp();
    }
    UACPI_STATUS_OK
}
