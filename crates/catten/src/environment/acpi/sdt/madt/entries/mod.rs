pub(super) mod interrupt_flags;
pub(super) mod interrupt_source_override;
pub mod ioapic;
pub(super) mod local_x2apic_nmi;
pub(super) mod nmi_source;

use alloc::vec::Vec;
use core::ptr::NonNull;

use spin::LazyLock;

use crate::cpu::isa::interface::memory::address::VirtualAddressIfce;
use crate::environment::acpi::table_map::{AcpiTableHeader, AcpiTableType, find_table_type};
use crate::memory::VirtualAddress;
use crate::memory::physical::PhysicalAddressIfce;

pub static MADT_INDEX: LazyLock<MadtEntryIndex> = LazyLock::new(|| {
    let madt_paddrs = find_table_type(AcpiTableType::MADT)
        .expect("[ACPI] PANIC: No MADT tables found on this ACPI based system.");
    if madt_paddrs.len() > 1 {
        panic!(
            "[ACPI] Warning: Multiple MADT tables found. Defaulting to using the first one though \
             this may be incorrect."
        );
    }
    unsafe { madt_paddrs[0].into_hhdm_ptr::<Madt>().as_ref_unchecked() }.parse()
});

const NUM_ENTRY_TYPES: usize = 28usize;

pub struct MadtEntryIndex {
    ptr_matrix: [Vec<*const MadtEntryGeneric>; NUM_ENTRY_TYPES],
}

impl MadtEntryIndex {
    pub fn get_entries_with_type(
        &self,
        entry_type: MadtEntryType,
    ) -> &Vec<*const MadtEntryGeneric> {
        &self.ptr_matrix[entry_type as usize]
    }
}

unsafe impl Sync for MadtEntryIndex {}
unsafe impl Send for MadtEntryIndex {}

#[derive(Debug)]
#[repr(C, packed)]
pub(super) struct MadtEntryGeneric {
    entry_type: u8,
    pub(super) entry_length: u8,
    // ...rest of the entry based on the specific type
}

unsafe impl Sync for MadtEntryGeneric {}

struct MadtEntryIter {
    ptr: Option<NonNull<MadtEntryGeneric>>,
    end_ptr: VirtualAddress,
}

impl MadtEntryIter {
    pub fn new(madt_ptr: *const Madt) -> Self {
        Self {
            ptr: unsafe {
                NonNull::new((madt_ptr as *const u8).byte_add(core::mem::size_of::<Madt>())
                    as *mut MadtEntryGeneric)
            },
            end_ptr: VirtualAddress::from_ptr(unsafe {
                (madt_ptr as *const u8).byte_add((*madt_ptr).header.length as usize)
            }),
        }
    }
}

impl Iterator for MadtEntryIter {
    type Item = NonNull<MadtEntryGeneric>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.ptr?;
        let start = VirtualAddress::from_ptr(current.as_ptr());
        if start + core::mem::size_of::<MadtEntryGeneric>() > self.end_ptr {
            self.ptr = None;
            return None;
        }
        let length = unsafe { current.read_unaligned() }.entry_length as usize;
        if length < core::mem::size_of::<MadtEntryGeneric>() || start + length > self.end_ptr {
            self.ptr = None;
            return None;
        }
        self.ptr = if start + length == self.end_ptr {
            None
        } else {
            NonNull::new(unsafe { current.as_ptr().byte_add(length) })
        };
        Some(current)
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Madt {
    header: AcpiTableHeader,
    lic_address: u32,
    flags: u32,
}

impl Madt {
    pub fn parse(&self) -> MadtEntryIndex {
        let mut ptr_matrix: [Vec<*const MadtEntryGeneric>; NUM_ENTRY_TYPES] = Default::default();
        let iter = MadtEntryIter::new(self);
        for entry_ptr in iter {
            let entry_type = unsafe { entry_ptr.as_ref() }.entry_type as usize;
            if entry_type < NUM_ENTRY_TYPES {
                ptr_matrix[entry_type].push(entry_ptr.as_ptr());
            }
        }
        MadtEntryIndex {
            ptr_matrix,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MadtEntryType {
    /* The Local APIC and all associated entry types are not used because this kernel requires
     * x86-64 based machines to support x2APIC mode. */
    _LocalApic = 0x0,
    IoApic = 0x1,
    InterruptSourceOverride = 0x2,
    NmiSource = 0x3,
    _LocalApicNmi = 0x4,
    _LocalApicAddressOverride = 0x5,
    /* The following three types are specific to IA-64, an ISA this kernel will never support
     * due to it being EOL. */
    _IoSapic = 0x6,
    _LocalSapic = 0x7,
    _PlatformInterruptSource = 0x8,
    ProcessorLocalX2Apic = 0x9,
    LocalX2ApicNmi = 0xa,
    /* The Aarch64 specific entries are only to be used on Aarch64 platforms. */
    GicCpuInterface = 0xb,
    GicDistributor = 0xc,
    GicMsiFrame = 0xd,
    GicRedistributor = 0xe,
    GicInterruptTranslationService = 0xf,
    /* The Limine MP feature is always used on all UEFI/ACPI platforms so this entry type is
     * never used. */
    _MultiprocessorWakeup = 0x10,
    /* The following seven entry types are never used by this kernel. */
    _CoreProgrammableInterruptController = 0x11,
    _LegacyIoProgrammableInterruptController = 0x12,
    _HyperTransportProgrammableInterruptController = 0x13,
    _ExtendIoProgrammableInterruptController = 0x14,
    _MsiProgrammableInterruptController = 0x15,
    _BridgeIoProgrammableInterruptController = 0x16,
    _LowPinCountProgrammableInterruptController = 0x17,
    /* The RISC-V specific entries are only to be used on RISC-V platforms. */
    RiscVHartLocalInterruptController = 0x18,
    RiscVIncomingMsiController = 0x19,
    RiscVAdvancedPlatformLevelInterruptController = 0x1a,
    RiscVPlatformLevelInterruptController = 0x1b,
}
