//! Run with `--features acpi_self_test` in kernel thread context.
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};

use uacpi_wrapper::*;

use crate::cpu::isa::lp::ops::{get_int_state, get_lp_id};
use crate::cpu::multiprocessor::interrupt_tracking::get_interrupt_depth;
use crate::logln;

static COMPLETED: AtomicUsize = AtomicUsize::new(0);
static SCI_COUNT: AtomicUsize = AtomicUsize::new(0);
static REMOTE_EVENT: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
static REMOTE_MUTEX: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
static REMOTE_DONE: AtomicBool = AtomicBool::new(false);

extern "C" fn remote_waiter() {
    assert_eq!(get_lp_id(), 1);
    unsafe {
        let mutex = REMOTE_MUTEX.load(Ordering::Acquire);
        assert_eq!(uacpi_kernel_acquire_mutex(mutex, 1000), UACPI_STATUS_OK);
        uacpi_kernel_sleep(2);
        assert_eq!(get_lp_id(), 1);
        uacpi_kernel_release_mutex(mutex);
        uacpi_kernel_signal_event(REMOTE_EVENT.load(Ordering::Acquire));
    }
    REMOTE_DONE.store(true, Ordering::Release);
}

unsafe extern "C" fn signal_later(event: uacpi_handle) {
    assert_eq!(get_lp_id(), 0);
    assert_eq!(get_interrupt_depth(), 0);
    unsafe {
        uacpi_kernel_sleep(2);
        COMPLETED.fetch_add(1, Ordering::Release);
        uacpi_kernel_signal_event(event);
    }
}

unsafe extern "C" fn report_sci(_: uacpi_handle) {
    assert_eq!(get_lp_id(), 0);
    assert_eq!(get_interrupt_depth(), 0);
    assert!(SCI_COUNT.load(Ordering::Acquire) > 0);
    logln!("[ACPI self-test] Power-button SCI and deferred work passed");
}

unsafe extern "C" fn power_button(_: uacpi_handle) -> uacpi_interrupt_ret {
    assert!(get_interrupt_depth() > 0);
    SCI_COUNT.fetch_add(1, Ordering::Release);
    assert_eq!(
        unsafe {
            uacpi_kernel_schedule_work(UACPI_WORK_NOTIFICATION, Some(report_sci), null_mut())
        },
        UACPI_STATUS_OK
    );
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

        let mutex = uacpi_kernel_create_mutex();
        assert!(!mutex.is_null());
        assert_eq!(uacpi_kernel_acquire_mutex(mutex, 0), UACPI_STATUS_OK);
        assert_eq!(uacpi_kernel_acquire_mutex(mutex, 0), UACPI_STATUS_TIMEOUT);
        let before = uacpi_kernel_get_nanoseconds_since_boot();
        assert_eq!(uacpi_kernel_acquire_mutex(mutex, 2), UACPI_STATUS_TIMEOUT);
        assert!(uacpi_kernel_get_nanoseconds_since_boot() - before >= 2_000_000);
        uacpi_kernel_release_mutex(mutex);
        assert_eq!(uacpi_kernel_acquire_mutex(mutex, u16::MAX), UACPI_STATUS_OK);
        uacpi_kernel_release_mutex(mutex);
        uacpi_kernel_free_mutex(mutex);

        let event = uacpi_kernel_create_event();
        assert!(!event.is_null());
        assert!(!uacpi_kernel_wait_for_event(event, 0));
        uacpi_kernel_signal_event(event);
        uacpi_kernel_signal_event(event);
        assert!(uacpi_kernel_wait_for_event(event, 0));
        assert!(uacpi_kernel_wait_for_event(event, 0));
        assert!(!uacpi_kernel_wait_for_event(event, 0));
        uacpi_kernel_signal_event(event);
        uacpi_kernel_reset_event(event);
        assert!(!uacpi_kernel_wait_for_event(event, 0));
        let before = uacpi_kernel_get_nanoseconds_since_boot();
        assert!(!uacpi_kernel_wait_for_event(event, 2));
        assert!(uacpi_kernel_get_nanoseconds_since_boot() - before >= 2_000_000);
        for kind in [UACPI_WORK_GPE_EXECUTION, UACPI_WORK_NOTIFICATION] {
            assert_eq!(
                uacpi_kernel_schedule_work(kind, Some(signal_later), event),
                UACPI_STATUS_OK
            );
            assert!(uacpi_kernel_wait_for_event(event, 1000));
        }
        assert_eq!(uacpi_kernel_wait_for_work_completion(), UACPI_STATUS_OK);
        assert_eq!(COMPLETED.load(Ordering::Acquire), 2);
        uacpi_kernel_free_event(event);

        if crate::cpu::multiprocessor::get_lp_count() > 1 {
            let event = uacpi_kernel_create_event();
            let mutex = uacpi_kernel_create_mutex();
            assert!(!event.is_null() && !mutex.is_null());
            assert_eq!(uacpi_kernel_acquire_mutex(mutex, 0), UACPI_STATUS_OK);
            REMOTE_EVENT.store(event, Ordering::Release);
            REMOTE_MUTEX.store(mutex, Ordering::Release);
            crate::cpu::scheduler::spawn_thread_on_lp(crate::memory::KERNEL_ASID, remote_waiter, 1);
            uacpi_kernel_sleep(2);
            uacpi_kernel_release_mutex(mutex);
            assert!(uacpi_kernel_wait_for_event(event, 1000));
            while !REMOTE_DONE.load(Ordering::Acquire) {
                crate::cpu::scheduler::yield_lp();
            }
            uacpi_kernel_free_event(event);
            uacpi_kernel_free_mutex(mutex);
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
