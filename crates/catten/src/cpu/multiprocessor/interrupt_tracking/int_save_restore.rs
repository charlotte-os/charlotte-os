use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};

use spin::LazyLock;

use crate::cpu::isa::lp::ops::{get_int_state, get_lp_id, mask_interrupts, unmask_interrupts};
use crate::cpu::multiprocessor::get_lp_count;

pub static INT_STATE: LazyLock<IntState> = LazyLock::new(|| IntState::new());

pub struct IntState {
    raw_locks: Vec<AtomicBool>,
    save_counts: Vec<usize>,
    saved_int_bits: Vec<bool>,
}

impl IntState {
    pub fn new() -> Self {
        let num_cpus = get_lp_count() as usize;
        Self {
            raw_locks: (0..num_cpus).map(|_| AtomicBool::default()).collect(),
            save_counts: vec![0; num_cpus],
            saved_int_bits: vec![false; num_cpus],
        }
    }

    pub fn save_int(&self) {
        let lp_idx = get_lp_id() as usize;
        let int_state = get_int_state();
        mask_interrupts!();
        // Spin until we can acquire the lock
        while self.raw_locks[lp_idx]
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            core::hint::spin_loop();
        }
        // Increment the save count using raw pointers and unsafe
        let sc_ptr = &raw const self.save_counts[lp_idx];
        // SAFETY: We want a mutable reference but Rust won't give one to us when self is not mut so
        // we use a raw internal lock and raw pointers to achieve the equivalent of interior
        // mutability. This is safe because we still use an AtomicBool to achieve mutual
        // exclusion just in a way that is too low level for rustc to understand.
        unsafe {
            let sc_mut = sc_ptr as *mut usize;
            *sc_mut += 1;
            // save and clear the interrupt enable bit if necessary.
            if self.save_counts[lp_idx] == 1 {
                let sib_ptr = &raw const self.saved_int_bits[lp_idx];
                let sib_mut = sib_ptr as *mut bool;
                *sib_mut = int_state;
            }
        }
        // Release the raw lock
        self.raw_locks[lp_idx].store(false, Ordering::Release);
    }

    pub fn restore_int(&self) {
        let lp_idx = get_lp_id() as usize;
        mask_interrupts!();
        // Spin until we can acquire the lock
        while self.raw_locks[lp_idx]
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            core::hint::spin_loop();
        }
        // Decrement the save count using raw pointers and unsafe
        let sc_ptr = &raw const self.save_counts[lp_idx];
        let mut restore_saved_int = false;
        unsafe {
            let sc_mut = sc_ptr as *mut usize;
            *sc_mut -= 1;
            // restore the interrupt enable bit if necessary.
            if self.save_counts[lp_idx] == 0 {
                let sib_ptr = &raw const self.saved_int_bits[lp_idx];
                let sib_mut = sib_ptr as *mut bool;
                restore_saved_int = *sib_mut;
            }
        }
        // Release the raw lock
        self.raw_locks[lp_idx].store(false, Ordering::Release);
        if restore_saved_int {
            unmask_interrupts!();
        }
    }
}
