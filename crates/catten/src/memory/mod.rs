//! # Memory Management Subsystem

pub mod allocators;
pub mod linear;
pub mod physical;

pub use linear::VirtualAddress;
pub use physical::{
    MemoryInterface,
    PhysicalAddress,
    PhysicalFrameAllocator,
};
pub use spin::LazyLock;

use crate::environment::boot_protocol::limine::{
    HHDM_REQUEST,
    MEMORY_MAP_REQUEST,
};
pub use crate::{
    cpu::{
        isa::{
            interface::memory::AddressSpaceInterface,
            memory::paging::AddressSpace,
        },
        multiprocessor::spin::{
            mutex::Mutex,
            rwlock::RwLock,
        },
    },
    klib::collections::id_table::IdTable,
};

pub type AddressSpaceId = usize;

/*The kernel address space is always ASID 0 and it is handled differently from userspace address
 * spaces because it needs to be initialized and accessible before the kernel allocator is
 * constructed and initialized.
 */
/// The kernel address space ID.
pub const KERNEL_ASID: AddressSpaceId = 0;
/// The kernel address space. It is initialized to the current address space when this static is
/// first accessed. Which should happen during the BSP init process.
pub static KERNEL_AS: LazyLock<Mutex<AddressSpace>> =
    LazyLock::new(|| Mutex::new(AddressSpace::get_current()));
/// Holds all userspace address spaces, indexed by their kernel assigned AddressSpaceId.
type AddressSpaceTable = IdTable<AddressSpace>;
pub static ADDRESS_SPACE_TABLE: LazyLock<AddressSpaceTable> = LazyLock::new(AddressSpaceTable::new);
/// The starting virtual address of the higher half direct mapping region created by the bootloader.
/// This should be remapped by the VMM during BSP init to be placed at the address specified by the
/// kernel virtual memory map at which point this address should be updated to reflect the new
/// location.
pub static HHDM_BASE: LazyLock<VirtualAddress> = LazyLock::new(|| {
    VirtualAddress::from(
        HHDM_REQUEST
            .response()
            .expect("Limine failed to provide a higher half direct mapping region.")
            .offset as usize,
    )
});
/// The physical frame allocator instance used by the kernel.
pub static PHYSICAL_FRAME_ALLOCATOR: LazyLock<Mutex<PhysicalFrameAllocator>> =
    LazyLock::new(|| {
        Mutex::new(PhysicalFrameAllocator::from(
            MEMORY_MAP_REQUEST.response().expect("Limine failed to provide a memory map."),
        ))
    });
