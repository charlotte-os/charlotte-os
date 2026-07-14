use core::mem::{offset_of, transmute};

const INIT_KERNEL_STACK_PAGES: usize = 16;

use crate::cpu::isa::init::gdt::{USER_CODE_SELECTOR, USER_DATA_SELECTOR};
use crate::cpu::isa::interface::memory::address::VirtualAddressIfce;
use crate::cpu::isa::lp::ops::{kernel_thread_trampoline, user_trampoline};
use crate::cpu::isa::memory::paging::PAGE_SIZE;
use crate::klib::collections::id_table;
use crate::memory::allocators::stack_allocator::{allocate_stack, deallocate_stack};
use crate::memory::{ADDRESS_SPACE_TABLE, AddressSpaceId, KERNEL_AS, VirtualAddress};

/// # Interrupt stack frame structure for x86_64 architecture
/// Note: must be 16 byte aligned as per `AMD APM 8.9.3`
#[repr(C, align(16))]
struct UserEntryFrames {
    // yield_lp return frame
    cr3: u64,
    rflags_cpl0: u64,
    rip0: u64,
    // iretq return frame
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

impl UserEntryFrames {
    fn new(asp: AddressSpaceId, entry_point: u64, iretq_rsp: VirtualAddress, flags: u64) -> Self {
        UserEntryFrames {
            cr3: ADDRESS_SPACE_TABLE
                .get(asp)
                .expect("Address space not found when creating thread context.")
                .get_cr3(),
            rflags_cpl0: 0x2,
            rip0: unsafe {
                transmute::<*const unsafe extern "C" fn() -> !, u64>(
                    user_trampoline as *const unsafe extern "C" fn() -> !,
                )
            },
            rip: entry_point,
            cs: USER_CODE_SELECTOR as u64,
            rflags: flags,
            rsp: <VirtualAddress as Into<u64>>::into(iretq_rsp),
            ss: USER_DATA_SELECTOR as u64,
        }
    }

    fn push_to_stack(self, rsp: &mut VirtualAddress) {
        let new_rsp = *rsp - core::mem::size_of::<UserEntryFrames>();
        unsafe {
            let isf_ptr = new_rsp.into_mut::<UserEntryFrames>();
            isf_ptr.write(self);
        }
        *rsp = new_rsp;
    }
}

#[repr(C, align(16))]
struct KernelEntryFrame {
    cr3: u64,
    rflags: u64,
    callee_saved_regs: [u64; 6],
    rip: u64,
}

impl KernelEntryFrame {
    fn new(cr3: u64, entry_point: u64) -> Self {
        let mut callee_saved_regs = [0; 6];
        callee_saved_regs[3] = entry_point;
        KernelEntryFrame {
            cr3,
            rflags: 0x2,
            callee_saved_regs,
            rip: kernel_thread_trampoline as *const () as u64,
        }
    }

    fn push_to_stack(self, rsp: &mut VirtualAddress) {
        let new_rsp = *rsp - core::mem::size_of::<KernelEntryFrame>();
        unsafe {
            let kef_ptr = new_rsp.into_mut::<KernelEntryFrame>();
            kef_ptr.write(self);
        }
        *rsp = new_rsp;
    }
}

#[derive(Debug)]
pub enum Error {
    AddressSpaceNotFound,
    StackAllocError(crate::memory::allocators::stack_allocator::Error),
    IdTableError(id_table::Error),
}

impl From<crate::memory::allocators::stack_allocator::Error> for Error {
    fn from(err: crate::memory::allocators::stack_allocator::Error) -> Self {
        Error::StackAllocError(err)
    }
}

impl From<id_table::Error> for Error {
    fn from(err: id_table::Error) -> Self {
        Error::IdTableError(err)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ThreadContext {
    pub rsp_cpl0: u64,
    _kernel_stack_buf: VirtualAddress,
    _user_stack_buf: Option<VirtualAddress>,
}

impl Drop for ThreadContext {
    fn drop(&mut self) {
        if let Some(user_stack_buf) = self._user_stack_buf {
            deallocate_stack(user_stack_buf)
                .expect("Failed to deallocate user stack for thread context.");
        }
        deallocate_stack(self._kernel_stack_buf)
            .expect("Failed to deallocate kernel stack for thread context.");
    }
}

impl ThreadContext {
    pub fn create_user_thread_context(
        asid: AddressSpaceId,
        entry_point: extern "C" fn(),
    ) -> Result<Self, Error> {
        let flags: u64 = 0x202;
        let user_stack_buf = allocate_stack(INIT_KERNEL_STACK_PAGES)
            .expect("Failed to allocate user stack for thread context.");
        let user_stack_top = user_stack_buf + INIT_KERNEL_STACK_PAGES * PAGE_SIZE;
        let isf = UserEntryFrames::new(asid, entry_point as u64, user_stack_top, flags);
        let kernel_stack_buf = allocate_stack(INIT_KERNEL_STACK_PAGES)
            .expect("Failed to allocate kernel stack for thread context.");
        let mut kernel_stack_top = kernel_stack_buf + INIT_KERNEL_STACK_PAGES * PAGE_SIZE;
        isf.push_to_stack(&mut kernel_stack_top);
        Ok(ThreadContext {
            rsp_cpl0: <VirtualAddress as Into<u64>>::into(kernel_stack_top),
            _kernel_stack_buf: VirtualAddress::default(),
            _user_stack_buf: Some(user_stack_buf),
        })
    }

    pub fn create_kernel_thread_context(entry_point: extern "C" fn()) -> Result<Self, Error> {
        let kernel_stack_buf = allocate_stack(INIT_KERNEL_STACK_PAGES)
            .expect("Failed to allocate kernel stack for thread context.");
        let mut kernel_stack_top = kernel_stack_buf + INIT_KERNEL_STACK_PAGES * PAGE_SIZE;
        let ksf = KernelEntryFrame::new(KERNEL_AS.lock().get_cr3(), entry_point as u64);
        ksf.push_to_stack(&mut kernel_stack_top);
        Ok(ThreadContext {
            rsp_cpl0: <VirtualAddress as Into<u64>>::into(kernel_stack_top),
            _kernel_stack_buf: kernel_stack_buf,
            _user_stack_buf: None,
        })
    }
}

#[unsafe(no_mangle)]
pub static TC_RSP_CPL0_OFFSET: usize = offset_of!(ThreadContext, rsp_cpl0);
