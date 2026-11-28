use core::sync::atomic::{AtomicBool, Ordering};

use crate::{cpu::isa::lp::ops::{get_int_state, mask_interrupts}};

#[derive(Debug, Default)]
pub struct InterruptState {
    raw_lock: AtomicBool,
    spinlocks_held: u32,
    saved_interrupt_state: Option<bool>,
}

impl InterruptState {
    pub fn mask_and_save(&self) {
        let int_state = get_int_state();
        mask_interrupts!();
        while let Err(_) = self.raw_lock.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire) {
            core::hint::spin_loop();
        }
        let self_mut = unsafe {core::mem::transmute::<&InterruptState, &mut InterruptState>(self)};
        if self_mut.spinlocks_held == 0 {
            self_mut.saved_interrupt_state = Some(int_state);
        }
        self_mut.spinlocks_held += 1;
        drop(self_mut);
        self.raw_lock.store(false, Ordering::Release);
    }

    pub fn restore()
}
