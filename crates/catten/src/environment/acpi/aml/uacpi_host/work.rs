//! Allocation-free interrupt-side submission to persistent kernel worker threads.

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use uacpi_wrapper::*;

use super::sync::Event;
use crate::cpu::multiprocessor::interrupt_tracking::get_interrupt_depth;
use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::cpu::scheduler::{spawn_thread_on_lp, yield_lp};
use crate::memory::KERNEL_ASID;

const QUEUE_CAPACITY: usize = 256;

#[derive(Clone, Copy)]
struct Work {
    handler: unsafe extern "C" fn(uacpi_handle),
    context: usize,
}

struct Queue {
    items: [Option<Work>; QUEUE_CAPACITY],
    head: usize,
    len: usize,
}

impl Queue {
    const fn new() -> Self {
        Self {
            items: [None; QUEUE_CAPACITY],
            head: 0,
            len: 0,
        }
    }

    fn push(&mut self, work: Work) -> bool {
        if self.len == QUEUE_CAPACITY {
            return false;
        }
        self.items[(self.head + self.len) % QUEUE_CAPACITY] = Some(work);
        self.len += 1;
        true
    }

    fn pop(&mut self) -> Option<Work> {
        if self.len == 0 {
            return None;
        }
        let item = self.items[self.head].take();
        self.head = (self.head + 1) % QUEUE_CAPACITY;
        self.len -= 1;
        item
    }
}

struct Worker {
    queue: Mutex<Queue>,
    available: Event,
}

impl Worker {
    const fn new() -> Self {
        Self {
            queue: Mutex::new(Queue::new()),
            available: Event::new(),
        }
    }

    fn run(&self) -> ! {
        loop {
            self.available.wait(u16::MAX);
            let work = self.queue.lock().pop();
            if let Some(work) = work {
                unsafe { (work.handler)(work.context as uacpi_handle) };
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
pub unsafe extern "C" fn uacpi_kernel_schedule_work(
    work_type: uacpi_work_type,
    handler: uacpi_work_handler,
    context: uacpi_handle,
) -> uacpi_status {
    let Some(handler) = handler else {
        return UACPI_STATUS_INVALID_ARGUMENT;
    };
    let worker = match work_type {
        UACPI_WORK_GPE_EXECUTION => &GPE_WORKER,
        UACPI_WORK_NOTIFICATION => &NOTIFY_WORKER,
        _ => return UACPI_STATUS_INVALID_ARGUMENT,
    };
    if !INITIALIZED.load(Ordering::Acquire) {
        return UACPI_STATUS_INIT_LEVEL_MISMATCH;
    }
    {
        let mut queue = worker.queue.lock();
        if !queue.push(Work {
            handler,
            context: context as usize,
        }) {
            return UACPI_STATUS_OUT_OF_MEMORY;
        }
        OUTSTANDING.fetch_add(1, Ordering::Release);
    }
    worker.available.signal();
    UACPI_STATUS_OK
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_wait_for_work_completion() -> uacpi_status {
    if get_interrupt_depth() != 0 {
        return UACPI_STATUS_INVALID_ARGUMENT;
    }
    super::interrupts::wait_for_interrupts();
    while OUTSTANDING.load(Ordering::Acquire) != 0 {
        yield_lp();
    }
    UACPI_STATUS_OK
}
