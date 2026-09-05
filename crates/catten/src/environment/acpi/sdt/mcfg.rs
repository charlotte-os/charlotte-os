use alloc::vec::Vec;
use core::ptr::read_unaligned;

use crate::device_management::drivers::busses::pci_express::topology::PcieSegmentGroup;
use crate::environment::acpi::table_map::{AcpiTableHeader, AcpiTableType, TABLE_MAP};
use crate::logln;
use crate::memory::PhysicalAddress;
use crate::memory::physical::PhysicalAddressIfce;

const MCFG_HEADER_RESERVED_SIZE: usize = 8;
const MCFG_HEADER_SIZE: usize = core::mem::size_of::<AcpiTableHeader>() + MCFG_HEADER_RESERVED_SIZE;
const MCFG_ENTRY_SIZE: usize = core::mem::size_of::<McfgEntry>();

pub fn parse_mcfg() -> Vec<PcieSegmentGroup> {
    logln!("[ACPI] Parsing MCFG table...");
    TABLE_MAP
        .get(&AcpiTableType::MCFG)
        .map(|tables| {
            logln!("[ACPI] MCFG table found at the following address: {:?}", (tables[0]));
            let table_ptr = unsafe { tables[0].into_hhdm_ptr::<AcpiTableHeader>() };
            let mcfg_header_ref = unsafe { &*table_ptr };
            logln!("[ACPI] MCFG table header: {:?}", mcfg_header_ref);
            if mcfg_header_ref.validate() {
                logln!(
                    "[ACPI] Validated MCFG table header checksum. Proceeding with MCFG parsing."
                );
                let table_len = mcfg_header_ref.length as usize;
                logln!("[ACPI] MCFG table length: {} bytes", (table_len));
                let entry_count = (table_len - MCFG_HEADER_SIZE) / MCFG_ENTRY_SIZE;
                logln!("[ACPI] MCFG entry count: {}", (entry_count));
                let mut segments = Vec::with_capacity(entry_count);
                for i in 0..entry_count {
                    let entry_ptr =
                        unsafe { (table_ptr).byte_add(MCFG_HEADER_SIZE + i * MCFG_ENTRY_SIZE) }
                            as *const McfgEntry;
                    segments.push(parse_mcfg_entry(entry_ptr));
                }
                segments
            } else {
                logln!(
                    "[ACPI] Invalid MCFG table header checksum. Skipping MCFG parsing and \
                     assuming no PCIe segments exist."
                );
                Vec::new()
            }
        })
        .unwrap_or_default()
}

#[derive(Debug)]
#[repr(C, packed)]
struct McfgEntry {
    ecam_base: PhysicalAddress,
    pcie_segment_num: u16,
    start_bus_num: u8,
    end_bus_num: u8,
    _reserved: u32,
}

fn parse_mcfg_entry(entry: *const McfgEntry) -> PcieSegmentGroup {
    logln!("[ACPI] Parsing MCFG entry at address {:p}", entry);
    let entry_data = unsafe { read_unaligned(entry) };
    logln!("[ACPI] MCFG entry data: {:?}", entry_data);

    PcieSegmentGroup::new(
        entry_data.pcie_segment_num,
        entry_data.ecam_base,
        entry_data.start_bus_num,
        entry_data.end_bus_num,
    )
}
