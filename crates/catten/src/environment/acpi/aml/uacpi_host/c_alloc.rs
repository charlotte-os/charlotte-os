use core::alloc::Layout;
use core::ffi::c_void;

use ft_wrapper::max_align_t;
use hashbrown::HashMap;
use spin::LazyLock;

use crate::cpu::isa::interface::memory::address::VirtualAddressIfce;
use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::memory::VirtualAddress;
use crate::memory::allocators::global_allocator::PRIMARY_ALLOCATOR;

static ALLOCATION_TABLE: LazyLock<Mutex<HashMap<VirtualAddress, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

const MALLOC_ALIGN: usize = core::mem::align_of::<max_align_t>();

pub extern "C" fn malloc(size: usize) -> *mut c_void {
    let layout = Layout::from_size_align(size, MALLOC_ALIGN).unwrap();

    if let Some(nnptr) = unsafe { PRIMARY_ALLOCATOR.lock().allocate(layout) } {
        let ptr = nnptr.as_ptr();
        ALLOCATION_TABLE.lock().insert(VirtualAddress::from_mut(ptr), size);
        ptr as *mut c_void
    } else {
        core::ptr::null_mut()
    }
}

pub extern "C" fn free(ptr: *mut c_void) {
    if !ptr.is_null() {
        let va = VirtualAddress::from_mut(ptr);
        if let Some(size) = ALLOCATION_TABLE.lock().remove(&va) {
            let layout = Layout::from_size_align(size, MALLOC_ALIGN).unwrap();
            unsafe { PRIMARY_ALLOCATOR.lock().deallocate(ptr.cast(), layout) };
        }
    }
}
