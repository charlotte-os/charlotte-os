use crate::cpu::isa::interface::memory::AddressSpaceInterface;
use crate::cpu::isa::interface::memory::address::VirtualAddressIfce;
use crate::cpu::isa::memory::paging::AddressSpace;
use crate::logln;
use crate::memory::PHYSICAL_FRAME_ALLOCATOR;
use crate::memory::linear::{MemoryMapping, PageType, VirtualAddress};

pub fn test_vmem() {
    let hhdm = 0xffff8000003ffff8usize as *const usize;
    macro_rules! probe {
        ($w:expr) => {
            crate::early_logln!(
                "[HEAPDBG] vmem {} phys0x3ffff8={:#x}",
                $w,
                (unsafe { hhdm.read() })
            );
        };
    }
    logln!("Entering Virtual Memory Subsystem Self Test");
    probe!("enter");
    logln!("Allocating physical frame");
    let frame = PHYSICAL_FRAME_ALLOCATOR.lock().allocate_frame().unwrap();
    crate::early_logln!(
        "[HEAPDBG] vmem frame={:#x}",
        (<crate::memory::PAddr as Into<usize>>::into(frame.clone()))
    );
    logln!("Physical frame allocated");
    logln!("Obtaining current address space");
    let mut current_as = AddressSpace::get_current();
    logln!("Obtained current address space.");
    logln!("Creating MemoryMapping struct.");
    let higher_half_start: VirtualAddress = VirtualAddress::from(0xffff_ffff_ffff_f000usize);
    let mapping = MemoryMapping {
        vaddr:     higher_half_start,
        paddr:     frame,
        page_type: PageType::KernelData,
    };
    logln!(
        "Created MemoryMapping struct.\nMapping the allocated frame to the beginning of the \
         higher half."
    );
    match current_as.map_page(mapping) {
        Ok(_) => logln!("Page mapped successfully."),
        Err(e) => panic!("Error mapping page: {:?}", e),
    }
    probe!("after map_page");
    let addr: *mut u32 = higher_half_start.into_mut();
    const MAGIC_NUMBER: u32 = 0xcafebabe;
    unsafe {
        logln!(
            "Writing magic number {:x?}_16 to virtual address {:?}",
            MAGIC_NUMBER,
            higher_half_start
        );
        addr.write(MAGIC_NUMBER);
        probe!("after write");
        logln!("Reading magic number back from {:?}", higher_half_start);
        let read_value = addr.read();
        assert_eq!(read_value, MAGIC_NUMBER);
        logln!("Magic number matches.");
        logln!("Test completed successfully.");
        logln!("Unmapping test page.");
        current_as.unmap_page(higher_half_start).expect("Error unmapping page.");
        probe!("after unmap_page");
        logln!("Test page successfully unmapped.");
        logln!("All virtual memory tests passed!");
    }
}
