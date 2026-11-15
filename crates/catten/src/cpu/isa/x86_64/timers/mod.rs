pub mod apic_timer;
pub mod tsc;

pub use apic_timer::LpTimer;

use crate::cpu::isa::interface::system_info::CpuInfoIfce;
use crate::cpu::isa::system_info::CpuInfo;
use crate::cpu::isa::timers::tsc::{IS_TSC_INVARIANT, TSC_CYCLE_PERIOD, TSC_FREQUENCY_HZ};
use crate::log::logln;

pub fn print_timer_info() {
    if *IS_TSC_INVARIANT {
        logln!("The x86-64 Timestamp Counter IS invariant.");
    } else {
        logln!("The x86-64 Timestamp Counter is NOT invariant.");
    }
    logln!("The x86-64 Timestamp Counter frequency is {:?} Hz.", (*TSC_FREQUENCY_HZ));
    logln!(
        "The x86-64 Timestamp Counter period is {:?} picoseconds.",
        ((*TSC_CYCLE_PERIOD).as_picos())
    );
    if CpuInfo::is_extension_supported(<CpuInfo as CpuInfoIfce>::IsaExtension::TscDeadline) {
        logln!("The x86-64 CPU supports TSC Deadline mode.");
    } else {
        logln!("The x86-64 CPU does NOT support TSC Deadline mode.");
    }
}
