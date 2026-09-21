use core::sync::atomic::{AtomicU64, Ordering};

use uacpi_wrapper::{uacpi_u8, uacpi_u64};

use crate::cpu::isa::interface::timers::LpTimerIfce;
use crate::cpu::isa::timers::LpTimer;
use crate::klib::time::duration::ExtDuration;

static LAST_NANOSECONDS: AtomicU64 = AtomicU64::new(0);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_get_nanoseconds_since_boot() -> uacpi_u64 {
    let now = LpTimer::timestamp_to_nanos(LpTimer::now());
    // Clamp cross-processor clock skew and repeated reads to a strictly
    // increasing value. Use the counter's full precision before conversion.
    let previous = LAST_NANOSECONDS
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |previous| {
            Some(now.max(previous.saturating_add(1)))
        })
        .unwrap();
    now.max(previous.saturating_add(1))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_stall(usec: uacpi_u8) {
    let start = LpTimer::now();
    let period = LpTimer::get_ts_cycle_period().as_picos();
    let ticks = (usec as u128 * 1_000_000).div_ceil(period) as u64;
    while LpTimer::now().wrapping_sub(start) < ticks {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_sleep(msec: uacpi_u64) {
    crate::cpu::scheduler::sleep(ExtDuration::from_millis(msec as u128));
}
