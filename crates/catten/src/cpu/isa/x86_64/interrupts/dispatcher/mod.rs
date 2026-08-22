use alloc::boxed::Box;
use core::{
    mem::MaybeUninit,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

use super::Error;

#[repr(C)]
pub struct InterruptEntryFrame {
    // The System V ABI requires 9 of the GPRs to be caller-saved.
    caller_saved_regs: [u64; 9],
    _padding: [u8; 3],
    vector_number: u8,
    error_code: u64,
    // interrupt return frame
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

static GLOBAL_INTERRUPT_DISPATCHER: InterruptDispatcher = InterruptDispatcher::new();

pub struct InterruptDispatcher {
    eh_table: [AtomicUsize; 32],
    dyn_ih_table: MaybeUninit<Box<[AtomicUsize]>>,
}

impl InterruptDispatcher {
    const fn new() -> Self {
        Self {
            eh_table: [const { AtomicUsize::new(0) }; 32],
            dyn_ih_table: MaybeUninit::uninit(),
        }
    }

    pub fn get() -> &'static Self {
        &GLOBAL_INTERRUPT_DISPATCHER
    }

    pub fn install_dyn_ih(
        &self,
        index: usize,
        handler: fn(&InterruptEntryFrame),
    ) -> Result<(), Error> {
        if let Err(_) = unsafe {
            (*self.dyn_ih_table.assume_init_ref())[index].compare_exchange(
                0,
                handler as usize,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
        } {
            Err(Error::DynIhIdxInUse(index))
        } else {
            Ok(())
        }
    }

    pub fn uninstall_dyn_ih(&self, index: usize) -> Result<(), Error> {
        unsafe {
            if let Some(entry) = (*self.dyn_ih_table.assume_init_ref()).get(index) {
                entry.store(0, Ordering::Release);
                Ok(())
            } else {
                Err(Error::DynIhIdxInvalid(index))
            }
        }
    }
}
