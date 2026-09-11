//! # RISC-V Thread Contexts

use crate::klib::collections::id_table;
use crate::memory::AddressSpaceId;
use crate::memory::allocators::stack_allocator;

#[derive(Debug)]
pub enum Error {
    AddressSpaceNotFound,
    StackAllocError(stack_allocator::Error),
    IdTableError(id_table::Error),
}

impl From<stack_allocator::Error> for Error {
    fn from(err: stack_allocator::Error) -> Self {
        Error::StackAllocError(err)
    }
}

impl From<id_table::Error> for Error {
    fn from(err: id_table::Error) -> Self {
        Error::IdTableError(err)
    }
}

/// The saved state of a thread on a hart.
///
/// The scheduler reaches straight into `kernel_stack_buf.curr_sp` when switching, so the field has
/// to exist and be laid out the same way it is on x86_64 even while the switch itself is stubbed.
#[derive(Debug)]
pub struct ThreadContext {
    pub kernel_stack_buf: stack_allocator::StackBuf,
}

impl ThreadContext {
    pub fn create_user_thread_context(
        _asid: AddressSpaceId,
        _entry_point: extern "C" fn(),
    ) -> Result<Self, Error> {
        todo!("Build the RISC-V U-mode entry frame (sepc, sstatus.SPP/SPIE, satp) on a new stack.")
    }

    pub fn create_kernel_thread_context(_entry_point: extern "C" fn()) -> Result<Self, Error> {
        todo!("Build the RISC-V S-mode entry frame on a freshly allocated kernel stack.")
    }
}
