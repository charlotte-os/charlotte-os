//! # Linear Address Map

use spin::LazyLock;

use super::VirtualAddress;
use crate::cpu::isa::memory::address::VADDR_SIG_BITS;
use crate::klib::size::*;

/// The rest of the kernel only sees the correct linear address map for the system it is running on
pub static LA_MAP: LazyLock<&'static LinearAddressMap> = LazyLock::new(|| match *VADDR_SIG_BITS {
    39 => &LA_MAP_39BIT,
    48 => &LA_MAP_48BIT,
    57 => &LA_MAP_57BIT,
    _ => panic!("Unsupported virtual address size"),
});

static LA_MAP_39BIT: LazyLock<LinearAddressMap> = LazyLock::new(|| LinearAddressMap {
    null_page: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000_0000_0000_0000usize),
        length: kibibytes(4),
    },
    application: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000000000001000usize),
        length: gibibytes(512),
    },
    direct_mapping: LinearMemoryRegion {
        base: VirtualAddress::from(0xffffff8000000000usize),
        length: gibibytes(512),
    },
    kernel_stack_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xffffff0000000000usize),
        length: gibibytes(4),
    },
    kernel_mmio: LinearMemoryRegion {
        base: VirtualAddress::from(0xffffff0800000000usize),
        length: gibibytes(4),
    },
    kernel_allocator_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xffffff1000000000usize),
        length: gibibytes(988),
    },
    kernel_image: LinearMemoryRegion {
        base: VirtualAddress::from(0xffffffff80000000usize),
        length: gibibytes(2),
    },
});

static LA_MAP_48BIT: LazyLock<LinearAddressMap> = LazyLock::new(|| LinearAddressMap {
    null_page: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000_0000_0000_0000usize),
        length: kibibytes(4),
    },
    application: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000000000001000usize),
        length: tebibytes(256),
    },
    direct_mapping: LinearMemoryRegion {
        base: VirtualAddress::from(0xffffff8000000000usize),
        length: tebibytes(256),
    },
    kernel_stack_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff810000000000usize),
        length: tebibytes(1),
    },
    kernel_mmio: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff820000000000usize),
        length: tebibytes(2),
    },
    kernel_allocator_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff840000000000usize),
        length: tebibytes(506),
    },
    kernel_image: LinearMemoryRegion {
        base: VirtualAddress::from(0xffffffff80000000usize),
        length: gibibytes(2),
    },
});

static LA_MAP_57BIT: LazyLock<LinearAddressMap> = LazyLock::new(|| LinearAddressMap {
    null_page: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000_0000_0000_0000usize),
        length: kibibytes(4),
    },
    application: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000000000001000usize),
        length: pebibytes(128),
    },
    direct_mapping: LinearMemoryRegion {
        base: VirtualAddress::from(0xffffff8000000000usize),
        length: pebibytes(128),
    },
    kernel_stack_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xff80000000000000usize),
        length: pebibytes(1),
    },
    kernel_mmio: LinearMemoryRegion {
        base: VirtualAddress::from(0xff88000000000000usize),
        length: pebibytes(1),
    },
    kernel_allocator_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xff90000000000000usize),
        length: pebibytes(253),
    },
    kernel_image: LinearMemoryRegion {
        base: VirtualAddress::from(0xffffffff80000000usize),
        length: gibibytes(2),
    },
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    NullPage,
    Application,
    DirectMapping,
    KernelStackArena,
    KernelMmio,
    KernelAllocatorArena,
    KernelImage,
}

pub struct LinearAddressMap {
    null_page: LinearMemoryRegion,
    application: LinearMemoryRegion,
    direct_mapping: LinearMemoryRegion,
    kernel_stack_arena: LinearMemoryRegion,
    kernel_mmio: LinearMemoryRegion,
    kernel_allocator_arena: LinearMemoryRegion,
    kernel_image: LinearMemoryRegion,
}

impl LinearAddressMap {
    pub fn region_type(&self, addr: VirtualAddress) -> RegionType {
        if self.null_page.contains(addr) {
            RegionType::NullPage
        } else if self.application.contains(addr) {
            RegionType::Application
        } else if self.direct_mapping.contains(addr) {
            RegionType::DirectMapping
        } else if self.kernel_stack_arena.contains(addr) {
            RegionType::KernelStackArena
        } else if self.kernel_mmio.contains(addr) {
            RegionType::KernelMmio
        } else if self.kernel_allocator_arena.contains(addr) {
            RegionType::KernelAllocatorArena
        } else if self.kernel_image.contains(addr) {
            RegionType::KernelImage
        } else {
            unreachable!(
                "This should be unreachable because the entire address space is tightly mapped \
                 and the VAddr type guarantees a valid address."
            );
        }
    }

    pub fn get_region(&self, region: RegionType) -> &LinearMemoryRegion {
        match region {
            RegionType::NullPage => &self.null_page,
            RegionType::Application => &self.application,
            RegionType::DirectMapping => &self.direct_mapping,
            RegionType::KernelStackArena => &self.kernel_stack_arena,
            RegionType::KernelMmio => &self.kernel_mmio,
            RegionType::KernelAllocatorArena => &self.kernel_allocator_arena,
            RegionType::KernelImage => &self.kernel_image,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinearMemoryRegion {
    pub base: VirtualAddress,
    pub length: usize,
}

impl LinearMemoryRegion {
    pub fn contains(&self, addr: VirtualAddress) -> bool {
        addr >= self.base && addr < (self.base + self.length)
    }
}

impl Into<(VirtualAddress, VirtualAddress)> for LinearMemoryRegion {
    fn into(self) -> (VirtualAddress, VirtualAddress) {
        (self.base, self.base + self.length)
    }
}
