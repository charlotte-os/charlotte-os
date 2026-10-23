use crate::cpu::isa::interface::memory::address::VirtualAddressIfce;
use crate::memory::allocators::memory::PageSize;
use crate::memory::linear::{MemoryMapping, PageType};
use crate::memory::physical::PhysicalAddressIfce;
use crate::memory::{AddressSpaceInterface, KERNEL_AS, PhysicalAddress, VirtualAddress};

pub unsafe fn punch_out_frame(frame: PhysicalAddress) {
    let hhdm_vaddr = VirtualAddress::from_ptr(unsafe { frame.into_hhdm_ptr::<u8>() });
    KERNEL_AS.lock().unmap_page(hhdm_vaddr).expect("Error unmapping page during HHDM punch out.");
}

pub unsafe fn punch_out_range(base: PhysicalAddress, frame_count: usize) {
    for i in 0..frame_count {
        let frame = base + i * PageSize::Standard.num_bytes();
        unsafe {
            punch_out_frame(frame);
        }
    }
}

pub unsafe fn punch_in_frame(frame: PhysicalAddress) {
    let hhdm_vaddr = VirtualAddress::from_ptr(unsafe { frame.into_hhdm_ptr::<u8>() });
    let mapping = MemoryMapping {
        vaddr: hhdm_vaddr,
        paddr: frame,
        page_type: PageType::KernelData,
    };
    KERNEL_AS.lock().map_page(mapping).expect("Error mapping page during HHDM punch in.");
}

pub unsafe fn punch_in_range(base: PhysicalAddress, frame_count: usize) {
    for i in 0..frame_count {
        let frame = base + i * PageSize::Standard.num_bytes();
        unsafe {
            punch_in_frame(frame);
        }
    }
}
