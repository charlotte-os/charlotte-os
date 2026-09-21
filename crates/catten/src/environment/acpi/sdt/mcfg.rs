//! MCFG discovery is deliberately independent of PCI device enumeration. AML can
//! access PCI configuration while the namespace and device topology are loading.
use alloc::vec::Vec;
use core::ptr::read_unaligned;

use spin::LazyLock;

use crate::cpu::isa::interface::memory::address::Address;
use crate::device_management::drivers::busses::pci_express::topology::PcieSegmentGroup;
use crate::environment::acpi::table_map::{AcpiTableHeader, AcpiTableType, TABLE_MAP};
use crate::logln;
use crate::memory::PhysicalAddress;
use crate::memory::physical::PhysicalAddressIfce;

const MCFG_HEADER_SIZE: usize = core::mem::size_of::<AcpiTableHeader>() + 8;
const MCFG_ENTRY_SIZE: usize = core::mem::size_of::<McfgEntry>();

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct McfgEntry {
    ecam_base: u64,
    pcie_segment_num: u16,
    start_bus_num: u8,
    end_bus_num: u8,
    _reserved: u32,
}

static MCFG_REGIONS: LazyLock<Vec<McfgEntry>> = LazyLock::new(|| {
    let mut regions = Vec::new();
    let Some(tables) = TABLE_MAP.get(&AcpiTableType::MCFG) else {
        return regions;
    };
    for table in tables {
        let table_ptr = unsafe { table.into_hhdm_ptr::<AcpiTableHeader>() };
        let header = unsafe { &*table_ptr };
        let length = header.length as usize;
        if length < MCFG_HEADER_SIZE
            || (length - MCFG_HEADER_SIZE) % MCFG_ENTRY_SIZE != 0
            || !header.validate()
        {
            logln!("[ACPI] Ignoring invalid MCFG table at {:?}", table);
            continue;
        }
        for offset in (MCFG_HEADER_SIZE..length).step_by(MCFG_ENTRY_SIZE) {
            let entry = unsafe { read_unaligned(table_ptr.byte_add(offset).cast::<McfgEntry>()) };
            let last = entry.ecam_base.checked_add(((entry.end_bus_num as u64 + 1) << 20) - 1);
            if entry.start_bus_num > entry.end_bus_num
                || entry.ecam_base % (1 << 20) != 0
                || !last.is_some_and(|address| {
                    usize::try_from(address).is_ok_and(PhysicalAddress::is_valid)
                })
            {
                logln!("[ACPI] Ignoring invalid MCFG allocation {:?}", entry);
                continue;
            }
            regions.push(entry);
        }
    }
    regions
});

pub fn parse_mcfg() -> Vec<PcieSegmentGroup> {
    MCFG_REGIONS
        .iter()
        .map(|entry| {
            PcieSegmentGroup::new(
                entry.pcie_segment_num,
                PhysicalAddress::from(entry.ecam_base),
                entry.start_bus_num,
                entry.end_bus_num,
            )
        })
        .collect()
}

/// Return a function's physical ECAM address without constructing DEVICE_TOPOLOGY.
/// Per the PCI firmware specification, MCFG bases are relative to bus zero even
/// when the allocation's first valid bus number is nonzero.
/// https://www.kernel.org/doc/html/v6.6/PCI/acpi-info.html
pub fn configuration_address(segment: u16, bus: u8, device: u8, function: u8) -> Option<u64> {
    if device >= 32 || function >= 8 {
        return None;
    }
    MCFG_REGIONS
        .iter()
        .find(|entry| {
            entry.pcie_segment_num == segment
                && (entry.start_bus_num..=entry.end_bus_num).contains(&bus)
        })
        .map(|entry| {
            entry.ecam_base
                + ((bus as u64) << 20)
                + ((device as u64) << 15)
                + ((function as u64) << 12)
        })
}
