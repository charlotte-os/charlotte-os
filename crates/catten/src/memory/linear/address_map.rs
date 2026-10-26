//! # Linear Address Map

use spin::LazyLock;

use super::VirtualAddress;
use crate::cpu::isa::interface::memory::address::Address;
use crate::cpu::isa::memory::address::VADDR_SIG_BITS;
use crate::klib::size::*;

/// The rest of the kernel only sees the correct linear address map for the system it is running on
pub static LA_MAP: LazyLock<&'static LinearAddressMap> = LazyLock::new(|| {
    let map: &'static LinearAddressMap = match *VADDR_SIG_BITS {
        39 => &LA_MAP_39BIT,
        48 => &LA_MAP_48BIT,
        57 => &LA_MAP_57BIT,
        _ => panic!("Unsupported virtual address size"),
    };
    map.assert_tiles_canonical_space(*VADDR_SIG_BITS);
    map
});

/* Each map tiles the canonical address space exactly. The lower half is
 * [0, 2^(n-1)) and the upper half is [2^64 - 2^(n-1), 2^64). The direct mapping takes the
 * bottom half of the upper half, and the kernel image occupies the top 2 GiB to comply with the
 * Limine Boot Protocol.
 */

/// 39 significant bits: 256 GiB per half.
static LA_MAP_39BIT: LazyLock<LinearAddressMap> = LazyLock::new(|| LinearAddressMap {
    null_page: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000_0000_0000_0000usize),
        length: kibibytes(4),
    },
    application: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000_0000_0000_1000usize),
        length: gibibytes(256) - kibibytes(4),
    },
    direct_mapping: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_ffc0_0000_0000usize),
        length: gibibytes(128),
    },
    kernel_stack_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_ffe0_0000_0000usize),
        length: gibibytes(4),
    },
    kernel_mmio: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_ffe1_0000_0000usize),
        length: gibibytes(2),
    },
    acpi_mappings: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_ffe1_8000_0000usize),
        length: gibibytes(2),
    },
    kernel_allocator_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_ffe2_0000_0000usize),
        length: gibibytes(118),
    },
    kernel_image: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_ffff_8000_0000usize),
        length: gibibytes(2),
    },
});

/// 48 significant bits: 128 TiB per half.
static LA_MAP_48BIT: LazyLock<LinearAddressMap> = LazyLock::new(|| LinearAddressMap {
    null_page: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000_0000_0000_0000usize),
        length: kibibytes(4),
    },
    application: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000_0000_0000_1000usize),
        length: tebibytes(128) - kibibytes(4),
    },
    direct_mapping: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_8000_0000_0000usize),
        length: tebibytes(64),
    },
    kernel_stack_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_c000_0000_0000usize),
        length: tebibytes(1),
    },
    kernel_mmio: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_c100_0000_0000usize),
        length: tebibytes(1),
    },
    acpi_mappings: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_c200_0000_0000usize),
        length: tebibytes(1),
    },
    kernel_allocator_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_c300_0000_0000usize),
        length: tebibytes(61) - gibibytes(2),
    },
    kernel_image: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_ffff_8000_0000usize),
        length: gibibytes(2),
    },
});

/// 57 significant bits: 64 PiB per half.
static LA_MAP_57BIT: LazyLock<LinearAddressMap> = LazyLock::new(|| LinearAddressMap {
    null_page: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000_0000_0000_0000usize),
        length: kibibytes(4),
    },
    application: LinearMemoryRegion {
        base: VirtualAddress::from(0x0000_0000_0000_1000usize),
        length: pebibytes(64) - kibibytes(4),
    },
    direct_mapping: LinearMemoryRegion {
        base: VirtualAddress::from(0xff00_0000_0000_0000usize),
        length: pebibytes(32),
    },
    kernel_stack_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xff80_0000_0000_0000usize),
        length: pebibytes(1),
    },
    kernel_mmio: LinearMemoryRegion {
        base: VirtualAddress::from(0xff84_0000_0000_0000usize),
        length: tebibytes(512),
    },
    acpi_mappings: LinearMemoryRegion {
        base: VirtualAddress::from(0xff86_0000_0000_0000usize),
        length: tebibytes(512),
    },
    kernel_allocator_arena: LinearMemoryRegion {
        base: VirtualAddress::from(0xff88_0000_0000_0000usize),
        length: pebibytes(30) - gibibytes(2),
    },
    kernel_image: LinearMemoryRegion {
        base: VirtualAddress::from(0xffff_ffff_8000_0000usize),
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
    AcpiMappings,
    KernelAllocatorArena,
    KernelImage,
}

pub struct LinearAddressMap {
    null_page: LinearMemoryRegion,
    application: LinearMemoryRegion,
    direct_mapping: LinearMemoryRegion,
    kernel_stack_arena: LinearMemoryRegion,
    kernel_mmio: LinearMemoryRegion,
    acpi_mappings: LinearMemoryRegion,
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
        } else if self.acpi_mappings.contains(addr) {
            RegionType::AcpiMappings
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
            RegionType::AcpiMappings => &self.acpi_mappings,
            RegionType::KernelAllocatorArena => &self.kernel_allocator_arena,
            RegionType::KernelImage => &self.kernel_image,
        }
    }

    /// Panics unless the regions exactly cover both canonical halves without gaps or overlaps.
    fn assert_tiles_canonical_space(&self, sig_bits: u8) {
        let half = 1u128 << (sig_bits - 1);
        let tile = |regions: &[&LinearMemoryRegion], start: u128, end: u128| {
            let mut cursor = start;
            for region in regions {
                let base = <VirtualAddress as Into<usize>>::into(region.base) as u128;
                assert_eq!(base, cursor, "Linear address map region {region:?} is misplaced");
                cursor += region.length as u128;
            }
            assert_eq!(cursor, end, "Linear address map does not fill its canonical half");
        };
        tile(&[&self.null_page, &self.application], 0, half);
        tile(
            &[
                &self.direct_mapping,
                &self.kernel_stack_arena,
                &self.kernel_mmio,
                &self.acpi_mappings,
                &self.kernel_allocator_arena,
                &self.kernel_image,
            ],
            (1u128 << 64) - half,
            1u128 << 64,
        );
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinearMemoryRegion {
    pub base: VirtualAddress,
    pub length: usize,
}

impl LinearMemoryRegion {
    pub fn contains(&self, addr: VirtualAddress) -> bool {
        addr >= self.base && ((addr - self.base) as usize) < self.length
    }
}

impl Into<(VirtualAddress, VirtualAddress)> for LinearMemoryRegion {
    /// The exclusive end is computed without canonicalization because the end of the lower half
    /// is non-canonical and would otherwise be sign-extended into the upper half.
    fn into(self) -> (VirtualAddress, VirtualAddress) {
        let end = <VirtualAddress as Into<usize>>::into(self.base)
            .checked_add(self.length)
            .expect("Linear memory region extends to the top of the address space");
        (self.base, unsafe { VirtualAddress::from_unchecked(end) })
    }
}
