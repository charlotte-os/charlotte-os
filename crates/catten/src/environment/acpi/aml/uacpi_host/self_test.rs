//! Run with `--features acpi_self_test` in kernel thread context.
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};

use uacpi_wrapper::*;

use crate::cpu::isa::lp::ops::{get_int_state, get_lp_id};
use crate::cpu::multiprocessor::interrupt_tracking::get_interrupt_depth;
use crate::logln;

static COMPLETED: AtomicUsize = AtomicUsize::new(0);
static CHAINED: AtomicUsize = AtomicUsize::new(0);
static SCI_COUNT: AtomicUsize = AtomicUsize::new(0);
static SCI_WORK: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
static SCI_WORK_PENDING: AtomicBool = AtomicBool::new(false);
static REMOTE_EVENT: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
static REMOTE_LOCK: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
static REMOTE_DONE: AtomicBool = AtomicBool::new(false);

extern "C" fn remote_waiter() {
    assert_eq!(get_lp_id(), 1);
    unsafe {
        let lock = REMOTE_LOCK.load(Ordering::Acquire);
        assert_eq!(uacpi_kernel_wait_for_semaphore(lock, 1000), UACPI_STATUS_OK);
        uacpi_kernel_sleep(2);
        assert_eq!(get_lp_id(), 1);
        uacpi_kernel_signal_semaphore(lock);
        uacpi_kernel_signal_semaphore(REMOTE_EVENT.load(Ordering::Acquire));
    }
    REMOTE_DONE.store(true, Ordering::Release);
}

unsafe extern "C" fn signal_later(event: uacpi_handle) {
    assert_eq!(get_lp_id(), 0);
    assert_eq!(get_interrupt_depth(), 0);
    unsafe {
        uacpi_kernel_sleep(2);
        COMPLETED.fetch_add(1, Ordering::Release);
        uacpi_kernel_signal_semaphore(event);
    }
}

/// Reschedules its own work item, which is also its context, from inside the
/// handler until it has run three times.
unsafe extern "C" fn reschedule_self(item: uacpi_handle) {
    assert_eq!(get_lp_id(), 0);
    unsafe { uacpi_kernel_sleep(1) };
    if CHAINED.fetch_add(1, Ordering::AcqRel) + 1 < 3 {
        unsafe {
            uacpi_kernel_schedule_work(UACPI_WORK_NOTIFICATION, item, Some(reschedule_self), item)
        };
    }
}

unsafe extern "C" fn report_sci(_: uacpi_handle) {
    SCI_WORK_PENDING.store(false, Ordering::Release);
    assert_eq!(get_lp_id(), 0);
    assert_eq!(get_interrupt_depth(), 0);
    assert!(SCI_COUNT.load(Ordering::Acquire) > 0);
    logln!("[ACPI self-test] Power-button SCI and deferred work passed");
}

unsafe extern "C" fn power_button(_: uacpi_handle) -> uacpi_interrupt_ret {
    assert!(get_interrupt_depth() > 0);
    SCI_COUNT.fetch_add(1, Ordering::Release);
    // A work item must not be scheduled again until its handler has started.
    if SCI_WORK_PENDING.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_ok() {
        unsafe {
            uacpi_kernel_schedule_work(
                UACPI_WORK_NOTIFICATION,
                SCI_WORK.load(Ordering::Acquire),
                Some(report_sci),
                null_mut(),
            )
        };
    }
    UACPI_INTERRUPT_HANDLED
}

pub fn run() {
    logln!("[ACPI self-test] Checking host services");
    super::memory::self_test();
    super::io::self_test();
    super::pci::self_test();
    unsafe {
        assert!(uacpi_kernel_alloc(usize::MAX).is_null());
        uacpi_kernel_free(null_mut());
        let allocation = uacpi_kernel_alloc(31);
        assert!(!allocation.is_null());
        assert_eq!(allocation as usize % 16, 0);
        allocation.cast::<u8>().write_bytes(0xa5, 31);
        uacpi_kernel_free(allocation);

        let mut rsdp = 0;
        assert_eq!(uacpi_kernel_get_rsdp(&mut rsdp), UACPI_STATUS_OK);
        let mapping = uacpi_kernel_map(rsdp, 36);
        assert_ne!(mapping, super::memory::MAP_FAILED);
        assert_eq!(core::slice::from_raw_parts(mapping.cast::<u8>(), 8), b"RSD PTR ");
        uacpi_kernel_unmap(mapping, 36);
        assert_eq!(uacpi_kernel_map(u64::MAX, 2), super::memory::MAP_FAILED);

        #[cfg(target_arch = "x86_64")]
        {
            let mut first = null_mut();
            let mut second = null_mut();
            assert_eq!(uacpi_kernel_io_map(0x80, 4, &mut first), UACPI_STATUS_OK);
            assert_eq!(uacpi_kernel_io_map(0x80, 4, &mut second), UACPI_STATUS_OK);
            let mut value = 0;
            assert_eq!(uacpi_kernel_io_read32(first, 1, &mut value), UACPI_STATUS_INVALID_ARGUMENT);
            uacpi_kernel_io_unmap(first);
            uacpi_kernel_io_unmap(second);

            // Exercise ECAM configuration access on the PCIe host bridge.
            let mut pci = null_mut();
            assert_eq!(
                uacpi_kernel_pci_device_open(
                    uacpi_pci_address {
                        segment: 0,
                        bus: 0,
                        device: 0,
                        function: 0,
                    },
                    &mut pci
                ),
                UACPI_STATUS_OK
            );
            let (mut byte, mut word, mut dword) = (0, 0, 0);
            assert_eq!(uacpi_kernel_pci_read8(pci, 1, &mut byte), UACPI_STATUS_OK);
            assert_eq!(uacpi_kernel_pci_read16(pci, 2, &mut word), UACPI_STATUS_OK);
            assert_eq!(uacpi_kernel_pci_read32(pci, 0, &mut dword), UACPI_STATUS_OK);
            assert_ne!(dword, u32::MAX);
            assert_eq!(byte, (dword >> 8) as u8);
            assert_eq!(word, (dword >> 16) as u16);
            uacpi_kernel_pci_device_close(pci);

            // Q35 exposes only segment zero. Uncovered functions must fail,
            // rather than falling back to legacy configuration ports.
            assert_eq!(
                uacpi_kernel_pci_device_open(
                    uacpi_pci_address {
                        segment: u16::MAX,
                        bus: 0,
                        device: 0,
                        function: 0
                    },
                    &mut pci,
                ),
                UACPI_STATUS_NOT_FOUND
            );
            assert!(pci.is_null());
        }

        assert!(get_int_state());
        let spinlock = uacpi_kernel_create_spinlock();
        assert!(!spinlock.is_null());
        let flags = uacpi_kernel_lock_spinlock(spinlock);
        assert!(!get_int_state());
        let nested = uacpi_kernel_disable_interrupts();
        uacpi_kernel_restore_interrupts(nested);
        assert!(!get_int_state());
        uacpi_kernel_unlock_spinlock(spinlock, flags);
        assert!(get_int_state());
        uacpi_kernel_free_spinlock(spinlock);

        let before = uacpi_kernel_get_nanoseconds_since_boot();
        uacpi_kernel_stall(50);
        assert!(uacpi_kernel_get_nanoseconds_since_boot() - before >= 50_000);
        let thread = uacpi_kernel_get_thread_id();
        assert_ne!(thread as usize, usize::MAX);
        let before = uacpi_kernel_get_nanoseconds_since_boot();
        uacpi_kernel_sleep(2);
        assert!(uacpi_kernel_get_nanoseconds_since_boot() - before >= 2_000_000);
        assert_eq!(uacpi_kernel_get_thread_id(), thread);

        // A semaphore with one unit is how uACPI builds its locks.
        let lock = uacpi_kernel_create_semaphore(1);
        assert!(!lock.is_null());
        assert_eq!(uacpi_kernel_wait_for_semaphore(lock, 0), UACPI_STATUS_OK);
        assert_eq!(uacpi_kernel_wait_for_semaphore(lock, 0), UACPI_STATUS_TIMEOUT);
        let before = uacpi_kernel_get_nanoseconds_since_boot();
        assert_eq!(uacpi_kernel_wait_for_semaphore(lock, 2), UACPI_STATUS_TIMEOUT);
        assert!(uacpi_kernel_get_nanoseconds_since_boot() - before >= 2_000_000);
        uacpi_kernel_signal_semaphore(lock);
        assert_eq!(uacpi_kernel_wait_for_semaphore(lock, u16::MAX), UACPI_STATUS_OK);
        // Units may be given back with interrupts masked, under a spinlock.
        let spinlock = uacpi_kernel_create_spinlock();
        let flags = uacpi_kernel_lock_spinlock(spinlock);
        uacpi_kernel_signal_semaphore(lock);
        uacpi_kernel_unlock_spinlock(spinlock, flags);
        uacpi_kernel_free_spinlock(spinlock);
        assert_eq!(uacpi_kernel_wait_for_semaphore(lock, 0), UACPI_STATUS_OK);
        uacpi_kernel_signal_semaphore(lock);
        uacpi_kernel_free_semaphore(lock);

        // One created with no units is how uACPI builds its events.
        let event = uacpi_kernel_create_semaphore(0);
        assert!(!event.is_null());
        assert_eq!(uacpi_kernel_wait_for_semaphore(event, 0), UACPI_STATUS_TIMEOUT);
        uacpi_kernel_signal_semaphore(event);
        uacpi_kernel_signal_semaphore(event);
        assert_eq!(uacpi_kernel_wait_for_semaphore(event, 0), UACPI_STATUS_OK);
        assert_eq!(uacpi_kernel_wait_for_semaphore(event, 0), UACPI_STATUS_OK);
        assert_eq!(uacpi_kernel_wait_for_semaphore(event, 0), UACPI_STATUS_TIMEOUT);
        let before = uacpi_kernel_get_nanoseconds_since_boot();
        assert_eq!(uacpi_kernel_wait_for_semaphore(event, 2), UACPI_STATUS_TIMEOUT);
        assert!(uacpi_kernel_get_nanoseconds_since_boot() - before >= 2_000_000);

        // One work item serves consecutive pieces of work of either type.
        let item = uacpi_kernel_create_work_item();
        assert!(!item.is_null());
        for kind in [UACPI_WORK_GPE_EXECUTION, UACPI_WORK_NOTIFICATION] {
            uacpi_kernel_schedule_work(kind, item, Some(signal_later), event);
            assert_eq!(uacpi_kernel_wait_for_semaphore(event, 1000), UACPI_STATUS_OK);
        }
        assert_eq!(uacpi_kernel_wait_for_work_completion(), UACPI_STATUS_OK);
        assert_eq!(COMPLETED.load(Ordering::Acquire), 2);
        uacpi_kernel_free_semaphore(event);

        // Completion also covers work scheduled by the work being waited for.
        uacpi_kernel_schedule_work(UACPI_WORK_NOTIFICATION, item, Some(reschedule_self), item);
        assert_eq!(uacpi_kernel_wait_for_work_completion(), UACPI_STATUS_OK);
        assert_eq!(CHAINED.load(Ordering::Acquire), 3);
        uacpi_kernel_free_work_item(item);

        if crate::cpu::multiprocessor::get_lp_count() > 1 {
            let event = uacpi_kernel_create_semaphore(0);
            let lock = uacpi_kernel_create_semaphore(1);
            assert!(!event.is_null() && !lock.is_null());
            assert_eq!(uacpi_kernel_wait_for_semaphore(lock, 0), UACPI_STATUS_OK);
            REMOTE_EVENT.store(event, Ordering::Release);
            REMOTE_LOCK.store(lock, Ordering::Release);
            crate::cpu::scheduler::spawn_thread_on_lp(crate::memory::KERNEL_ASID, remote_waiter, 1);
            uacpi_kernel_sleep(2);
            uacpi_kernel_signal_semaphore(lock);
            assert_eq!(uacpi_kernel_wait_for_semaphore(event, 1000), UACPI_STATUS_OK);
            while !REMOTE_DONE.load(Ordering::Acquire) {
                crate::cpu::scheduler::yield_lp();
            }
            uacpi_kernel_free_semaphore(event);
            uacpi_kernel_free_semaphore(lock);
        }

        let mut revision = 0;
        assert_eq!(
            uacpi_eval_simple_integer(null_mut(), c"\\_REV".as_ptr(), &mut revision),
            UACPI_STATUS_OK
        );
        assert_eq!(revision, 2);
        logln!("[ACPI self-test] Host services and namespace evaluation passed");

        // An external QEMU `system_powerdown` command now exercises real SCI
        // delivery, IRQ-safe queue submission, and resumption of an idle worker.
        let mut reduced = false;
        assert_eq!(uacpi_is_platform_reduced_hardware(&mut reduced), UACPI_STATUS_OK);
        if !reduced {
            let item = uacpi_kernel_create_work_item();
            assert!(!item.is_null());
            SCI_WORK.store(item, Ordering::Release);
            assert_eq!(
                uacpi_install_fixed_event_handler(
                    UACPI_FIXED_EVENT_POWER_BUTTON,
                    Some(power_button),
                    null_mut()
                ),
                UACPI_STATUS_OK
            );
            assert_eq!(
                uacpi_uninstall_fixed_event_handler(UACPI_FIXED_EVENT_POWER_BUTTON),
                UACPI_STATUS_OK
            );
            assert_eq!(
                uacpi_install_fixed_event_handler(
                    UACPI_FIXED_EVENT_POWER_BUTTON,
                    Some(power_button),
                    null_mut()
                ),
                UACPI_STATUS_OK
            );
            logln!("[ACPI self-test] Ready for power-button SCI");
        }
    }
}
