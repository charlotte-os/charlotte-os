//! Physical mappings used during both early table discovery and AML execution.
use core::ffi::c_void;
use core::sync::atomic::{AtomicUsize, Ordering};

use uacpi_wrapper::{uacpi_phys_addr, uacpi_size};

use crate::cpu::isa::interface::memory::address::{Address, VirtualAddressIfce};
use crate::memory::allocators::memory::PageSize;
use crate::memory::linear::address_map::{LA_MAP, RegionType};
use crate::memory::linear::{MemoryMapping, PageType};
use crate::memory::{AddressSpaceInterface, KERNEL_AS, PhysicalAddress, VirtualAddress};

// The kernel has no synchronous cross-CPU TLB shootdown yet. Reserve a separate
// aperture and retire VAs permanently: no stale translation can alias a later
// mapping, and unmapping retains page-table pages that other CPUs may cache.
// Exhaustion returns MAP_FAILED instead of reusing an unsafe VA.
static NEXT_OFFSET: AtomicUsize = AtomicUsize::new(0);

pub(super) const MAP_FAILED: *mut c_void = usize::MAX as *mut c_void;

fn page_span(addr: usize, len: usize) -> Option<(usize, usize, usize)> {
    if len == 0 {
        return None;
    }
    let page_size = PageSize::Standard.num_bytes();
    let offset = addr % page_size;
    let span = len.checked_add(offset)?.checked_add(page_size - 1)? & !(page_size - 1);
    let base = addr - offset;
    base.checked_add(span - 1)?;
    Some((base, offset, span / page_size))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_map(addr: uacpi_phys_addr, len: uacpi_size) -> *mut c_void {
    let Ok(addr) = usize::try_from(addr) else {
        return MAP_FAILED;
    };
    let Some((base, offset, pages)) = page_span(addr, len) else {
        return MAP_FAILED;
    };
    let page_size = PageSize::Standard.num_bytes();
    if !PhysicalAddress::is_valid(base) || !PhysicalAddress::is_valid(base + pages * page_size - 1)
    {
        return MAP_FAILED;
    }
    let region = LA_MAP.get_region(RegionType::AcpiMappings);
    let bytes = pages * page_size;
    let Ok(offset_in_region) =
        NEXT_OFFSET.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |offset| {
            offset.checked_add(bytes).filter(|end| *end <= region.length)
        })
    else {
        return MAP_FAILED;
    };
    let virtual_base = region.base + offset_in_region;
    let mut kas = KERNEL_AS.lock();
    // Map individually so a failed request leaves no partial mappings behind.
    for page in 0..pages {
        if kas
            .map_page(MemoryMapping {
                vaddr: virtual_base + page * page_size,
                paddr: unsafe { PhysicalAddress::from_unchecked(base + page * page_size) },
                page_type: PageType::Mmio,
            })
            .is_err()
        {
            for mapped in 0..page {
                let _ = retire_page(&mut kas, virtual_base + mapped * page_size);
            }
            return MAP_FAILED;
        }
    }
    (virtual_base + offset).into_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_unmap(addr: *mut c_void, len: uacpi_size) {
    if addr.is_null() || addr == MAP_FAILED {
        return;
    }
    let Some((base, _, pages)) = page_span(addr as usize, len) else {
        return;
    };
    let virtual_base = VirtualAddress::from(base);
    let mut kas = KERNEL_AS.lock();
    for page in 0..pages {
        if retire_page(&mut kas, virtual_base + page * PageSize::Standard.num_bytes()).is_err() {
            crate::logln!("[ACPI][uACPI] Failed to release a physical mapping");
        }
    }
}

fn retire_page(
    kas: &mut crate::memory::AddressSpace,
    address: VirtualAddress,
) -> Result<PhysicalAddress, crate::cpu::isa::memory::Error> {
    #[cfg(target_arch = "x86_64")]
    {
        kas.unmap_page_retaining_tables(address)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        kas.unmap_page(address)
    }
}

pub(super) fn self_test() {
    assert_eq!(page_span(0x1abc, 0xf00), Some((0x1000, 0xabc, 2)));
    assert_eq!(page_span(0x1000, 0x1000), Some((0x1000, 0, 1)));
    assert_eq!(page_span(0x1fff, 2), Some((0x1000, 0xfff, 2)));
    assert_eq!(page_span(0, 0), None);
    assert_eq!(page_span(usize::MAX, 2), None);
    assert_eq!(page_span(0, usize::MAX), None);
}
