//! # RISC-V Timers
//!
//! The architectural timer is the `time` CSR paired with the SBI Timer extension (or Sstc's
//! `stimecmp` where it is available) for arming the next interrupt.

use alloc::sync::Arc;

use crate::cpu::isa::interface::timers::{ExtDuration, LpTimerError, LpTimerIfce};
use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::logln;

pub struct SupervisorTimer;

pub type LpTimer = SupervisorTimer;

impl LpTimerIfce for SupervisorTimer {
    type Divisor = u32;
    type IntDispatchNum = u8;
    type TickCount = u64;
    type Timestamp = u64;

    const NAME: &'static str = "RISC-V Supervisor Timer";

    fn get() -> Arc<Mutex<Self>> {
        todo!("Return this hart's timer once per-hart timer instances are constructed.")
    }

    fn now() -> Self::Timestamp {
        todo!("Read the `time` CSR, which counts at the platform's fixed timebase frequency.")
    }

    fn get_ts_cycle_period() -> ExtDuration {
        todo!("Derive the period from the timebase-frequency in the DT or the RHCT ACPI table.")
    }

    fn get_int_resolution(&self) -> Result<ExtDuration, LpTimerError> {
        todo!("Report the timer's resolution, which is one tick of the platform timebase.")
    }

    fn set_divisor(&mut self, _divisor: Self::Divisor) -> Result<(), LpTimerError> {
        // The RISC-V timebase is fixed by the platform and has no programmable divisor.
        Err(LpTimerError::DivisorNotSupported)
    }

    fn set_duration(&mut self, _duration: ExtDuration) -> Result<(), LpTimerError> {
        todo!("Convert the duration to ticks and arm `stimecmp` relative to the current `time`.")
    }

    fn set_deadline(&mut self, _deadline: Self::Timestamp) -> Result<(), LpTimerError> {
        todo!("Arm `stimecmp` directly, falling back to the SBI Timer extension without Sstc.")
    }

    fn get_duration(&self) -> Result<ExtDuration, LpTimerError> {
        todo!("Report the remaining time until the armed deadline.")
    }

    fn start(&mut self) -> Result<(), LpTimerError> {
        todo!("Unmask STIE in `sie` so the armed deadline can fire.")
    }

    fn stop(&mut self) -> Result<(), LpTimerError> {
        todo!("Mask STIE in `sie`.")
    }

    fn reset(&mut self) -> Result<(), LpTimerError> {
        todo!("Clear any armed deadline and return the timer to its idle state.")
    }

    fn get_interrupt_mask(&mut self) -> Result<bool, LpTimerError> {
        todo!("Report the STIE bit in `sie`.")
    }

    fn set_interrupt_mask(&mut self, _mask: bool) -> Result<(), LpTimerError> {
        todo!("Set or clear the STIE bit in `sie`.")
    }

    fn set_isr_dispatch_number(&mut self, _num: Self::IntDispatchNum) -> Result<(), LpTimerError> {
        // The timer interrupt is always reported as `scause` 5; it is not redirectable.
        Err(LpTimerError::TimerStartsAutomatically)
    }
}

pub fn print_timer_info() {
    logln!("The RISC-V supervisor timer has not been characterised yet.");
}
