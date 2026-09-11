//! # AArch64 Timers
//!
//! The Arm Generic Timer provides a system counter read through `CNTPCT_EL0`, running at the
//! frequency reported by `CNTFRQ_EL0`, and a per-LP EL1 physical timer armed through
//! `CNTP_CVAL_EL0` or `CNTP_TVAL_EL0`.

use alloc::sync::Arc;

use crate::cpu::isa::interface::timers::{ExtDuration, LpTimerError, LpTimerIfce};
use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::logln;

pub type LpTimer = ArmGenericTimer;

pub struct ArmGenericTimer;

impl LpTimerIfce for ArmGenericTimer {
    type Divisor = u32;
    type IntDispatchNum = u32;
    type TickCount = u64;
    type Timestamp = u64;

    const NAME: &'static str = "Arm Generic Timer";

    fn get() -> Arc<Mutex<Self>> {
        todo!("Return this LP's timer once per-LP timer instances are constructed.")
    }

    fn now() -> Self::Timestamp {
        todo!("Read CNTPCT_EL0, with an isb beforehand so the read is not speculated.")
    }

    fn get_ts_cycle_period() -> ExtDuration {
        todo!("Derive the period from CNTFRQ_EL0.")
    }

    fn get_int_resolution(&self) -> Result<ExtDuration, LpTimerError> {
        todo!("Report the timer's resolution, which is one tick of the system counter.")
    }

    fn set_divisor(&mut self, _divisor: Self::Divisor) -> Result<(), LpTimerError> {
        // The Generic Timer counts at a fixed frequency and has no programmable divisor.
        Err(LpTimerError::DivisorNotSupported)
    }

    fn set_duration(&mut self, _duration: ExtDuration) -> Result<(), LpTimerError> {
        todo!("Convert the duration to ticks and write it to CNTP_TVAL_EL0.")
    }

    fn set_deadline(&mut self, _deadline: Self::Timestamp) -> Result<(), LpTimerError> {
        todo!("Write the absolute deadline to CNTP_CVAL_EL0.")
    }

    fn get_duration(&self) -> Result<ExtDuration, LpTimerError> {
        todo!("Read CNTP_TVAL_EL0 and convert the remaining ticks to a duration.")
    }

    fn start(&mut self) -> Result<(), LpTimerError> {
        todo!("Set CNTP_CTL_EL0.ENABLE and clear IMASK.")
    }

    fn stop(&mut self) -> Result<(), LpTimerError> {
        todo!("Clear CNTP_CTL_EL0.ENABLE.")
    }

    fn reset(&mut self) -> Result<(), LpTimerError> {
        todo!("Disable the timer and clear any armed deadline.")
    }

    fn get_interrupt_mask(&mut self) -> Result<bool, LpTimerError> {
        todo!("Report CNTP_CTL_EL0.IMASK.")
    }

    fn set_interrupt_mask(&mut self, _mask: bool) -> Result<(), LpTimerError> {
        todo!("Set or clear CNTP_CTL_EL0.IMASK.")
    }

    fn set_isr_dispatch_number(&mut self, _num: Self::IntDispatchNum) -> Result<(), LpTimerError> {
        // The EL1 physical timer is wired to a fixed PPI and cannot be redirected.
        Err(LpTimerError::TimerStartsAutomatically)
    }
}

pub fn print_timer_info() {
    logln!("The Arm Generic Timer has not been characterised yet.");
}
