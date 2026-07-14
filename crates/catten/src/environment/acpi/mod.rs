//! The Advanced Configuration and Power Interface (ACPI) Subsystem
//!
//! The Advanced Configuration and Power Interface (ACPI) is an open industry specification that
//! defines a flexible and extensible interface for hardware discovery, configuration, power
//! management, and monitoring. ACPI provides a standardized way for the operating system to
//! interact with the underlying hardware, allowing it to manage power states, configure devices,
//! and perform other system-level tasks in a platform-independent manner.
//!
//! It provides information in two different forms:
//!
//! - System Description Tables (SDTs)
//! - ACPI Machine Language (AML) bytecode tables
//!
//! This module is split into two submodules, `sdt` and `aml`, which contain code for working with
//! each of these forms of information respectively and their inline documentation contains more
//! detailed information about how each of them works and how Catten uses them.
//!
//! This top level module contains code for finding and parsing the XSDT to find the physical
//! addresses of other ACPI tables, as well as some common data structures and utilities for working
//! with the headers that are common to all ACPI tables.
//!
//! The primary reference needed to understand and work with this module is the [ACPI specification](https://uefi.org/specs/ACPI/6.6/).
//! In addition to the specification the source code of [uACPI](https://github.com/uACPI/uACPI) a portable C language
//! ACPI implementation made to be robust enough to handle buggy firmware can be very helpful to
//! understand how to work with ACPI and handle various edge cases and quirks of real world
//! firmware.
//!
//! It should be noted however that the Catten kernel does not and will not integrate uACPI or any
//! other third party ACPI implementation. Accordingly this subsystem is to be developed entirely
//! independently in manually written Rust and in such a way as to be tightly integrated with the
//! rest of the kernel. Features will be added as they are needed and the implementation will
//! generally assume that target system firmware appropriately conforms to the latest published ACPI
//! specification.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::NonNull;

use hashbrown::HashMap;
use spin::LazyLock;

use crate::cpu::isa::interface::memory::address::Address;
use crate::environment::boot_protocol::limine::RSDP_REQUEST;
use crate::logln;
use crate::memory::PhysicalAddress;
use crate::memory::physical::PhysicalAddressIfce;

pub mod aml;
pub mod sdt;

static TABLE_MAP: LazyLock<HashMap<AcpiTableType, Vec<PhysicalAddress>>> = LazyLock::new(|| {
    if let Some(xsdp_ptr) = get_xsdp() {
        let xsdp: &Xsdp = unsafe { xsdp_ptr.as_ref() };
        if !xsdp.validate() {
            panic!("Invalid ACPI Extended System Description Pointer (XSDP)");
        }
        let xsdt_addr = PhysicalAddress::from(xsdp.xsdt_address);
        if xsdt_addr.is_null() {
            panic!("ACPI Extended System Description Table (XSDT) required but not found");
        }
        parse_xsdt(xsdt_addr)
    } else {
        panic!("ACPI not available");
    }
});

pub fn print_table_map() {
    let mut output = String::new();
    output.push_str("ACPI Table Map:\n");
    for (table_type, addrs) in TABLE_MAP.iter() {
        output.push_str(&format!("{:?}: {:?}\n", table_type, addrs));
    }
    logln!("{output}");
}

#[derive(Debug)]
pub enum Error {
    AcpiUnavailable,
    InvalidXsdp,
    XsdtNotFound,
    TableNotFound,
    InvalidTableSignature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AcpiTableType {
    RSDT, /* Root System Description Table, contains 32-bit physical addresses of other ACPI
           * tables => Unused by Catten since we require ACPI 2.0 or higher */
    XSDT, /* Extended System Description Table, contains 64-bit physical addresses of other ACPI tables => Used by Catten to find the physical addresses of other ACPI tables */
    FADT, /* Fixed ACPI Description Table, contains fixed hardware configuration information
           * and pointers to other tables like the DSDT => Used by Catten to find the physical
           * address of the DSDT and determine if the system supports 64-bit PM timer access
           * and other features */
    MADT, /* Multiple APIC Description Table, contains information about the system's
           * interrupt controllers => Used by Catten to configure platform level interrupt
           * controllers since processor local ones are usually more easily found by other
           * means */
    SRAT, /* System Resource Affinity Table, contains information about the system's NUMA nodes
           * and their proximity to CPUs and memory => Unused by Catten since we don't
           * currently have any NUMA-aware features planned. Will be used eventually when NUMA
           * support lands */
    MCFG, /* Memory-Mapped Configuration Space Base Address Description Table, contains
           * information about the system's PCI Express configuration spaces => Used by Catten
           * to find the physical address of all PCI Express configuration spaces and access
           * them to configure PCI Express devices */
    HPET, /* High Precision Event Timer Table, contains information about the system's HPET
           * timer => Currently unused given that all timing on x86_64 is done using the TSC
           * and APIC timers */
    WAET, /* Windows ACPI Emulated Devices Table, used by Windows to detect if it's running in
           * a virtual machine => Unused by Catten */
    BGRT, /* Boot Graphics Resource Table, used to pass a boot logo from the firmware to the OS
           * => Unused by Catten */
    IVRS, /* I/O Virtualization Reporting Structure, used by AMD's SVM => Used by Catten to
           * determine if AMD SVM is supported */
    UEFI, /* UEFI ACPI table, used by UEFI drivers to avoid name collisions with ACPI tables =>
           * Unused by Catten */
    TPM2, /* Trusted Platform Module 2.0 Table, used for TPM 2.0 support => Unused by Catten */
    MSDM, /* Microsoft Data Management Table, used for Windows Product Activation => Unused by
           * Catten */
    BOOT, /* Microsoft Boot Table, used for Windows boot configuration => Unused by Catten */
    SLIC, /* Software Licensing Description Table, used for Windows activation => Unused by
           * Catten */
    VFCT, /* Video Firmware Configuration Table, used for early VBIOS access particularly with
           * AMD GPUs => Unused so far, may be used in the future */
    CRAT, /* Coherent Resource Affinity Table, is used in systems with heterogeneous computing,
           * such as AMD APUs, to describe the topology, affinity, and coherence of memory and
           * processing units to the operating system => Not presently used, may be used in the
           * future */
    CDIT, /* Coherent Device Information Table, used to provide information about coherent devices to the operating system
          => Not presently used, may be used in the future */
    FPDT, /* Firmware Performance Data Table, used to provide performance data about the
           * firmware => Unused by Catten */
    WSMT, /* Windows SMM Security Mitigations Table, used to indicate support for various SMM
           * security mitigations => Unused by Catten, may be used in the future if possible */
    RHCT, /* RISC-V Hart Capabilities Table, used to describe the capabilities of RISC-V harts
           * => Used to query hart ISA functionality on RISC-V systems */
    GTDT, /* Generic Timer Description Table, contains information about the system's generic
           * timers on ARM systems => Used by Catten to find the physical address of the
           * generic timer */
    SPCR, /* Serial Port Console Redirection Table, used to describe a serial port that can be
           * used for console redirection => Unused by Catten, may be used in the
           * future for early boot logging */
    DSDT, /* Differentiated System Description Table, contains AML bytecode that describes the
           * system's devices and their configuration => Used to build and
           * use the ACPI Namespace and execute AML methods to configure devices and
           * perform other ACPI operations */
    SSDT, /* Secondary System Description Table, contains AML bytecode that describes
           * additional devices and configuration that doesn't fit in the DSDT =>
           * Used to build and use the ACPI Namespace and execute AML
           * methods to configure devices and perform other ACPI operations */
}

impl TryFrom<[u8; 4]> for AcpiTableType {
    type Error = Error;

    fn try_from(signature: [u8; 4]) -> Result<Self, Self::Error> {
        match signature {
            [b'R', b'S', b'D', b'T'] => Ok(Self::RSDT),
            [b'X', b'S', b'D', b'T'] => Ok(Self::XSDT),
            [b'F', b'A', b'C', b'P'] => Ok(Self::FADT),
            [b'A', b'P', b'I', b'C'] => Ok(Self::MADT),
            [b'S', b'R', b'A', b'T'] => Ok(Self::SRAT),
            [b'M', b'C', b'F', b'G'] => Ok(Self::MCFG),
            [b'H', b'P', b'E', b'T'] => Ok(Self::HPET),
            [b'W', b'A', b'E', b'T'] => Ok(Self::WAET),
            [b'B', b'G', b'R', b'T'] => Ok(Self::BGRT),
            [b'I', b'V', b'R', b'S'] => Ok(Self::IVRS),
            [b'U', b'E', b'F', b'I'] => Ok(Self::UEFI),
            [b'T', b'P', b'M', b'2'] => Ok(Self::TPM2),
            [b'M', b'S', b'D', b'M'] => Ok(Self::MSDM),
            [b'B', b'O', b'O', b'T'] => Ok(Self::BOOT),
            [b'S', b'L', b'I', b'C'] => Ok(Self::SLIC),
            [b'V', b'F', b'C', b'T'] => Ok(Self::VFCT),
            [b'C', b'R', b'A', b'T'] => Ok(Self::CRAT),
            [b'C', b'D', b'I', b'T'] => Ok(Self::CDIT),
            [b'F', b'P', b'D', b'T'] => Ok(Self::FPDT),
            [b'W', b'S', b'M', b'T'] => Ok(Self::WSMT),
            [b'R', b'H', b'C', b'T'] => Ok(Self::RHCT),
            [b'G', b'T', b'D', b'T'] => Ok(Self::GTDT),
            [b'S', b'P', b'C', b'R'] => Ok(Self::SPCR),
            [b'D', b'S', b'D', b'T'] => Ok(Self::DSDT),
            [b'S', b'S', b'D', b'T'] => Ok(Self::SSDT),
            _ => Err(Error::InvalidTableSignature),
        }
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct Xsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32, // deprecated since version 2.0

    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

impl Xsdp {
    fn validate(&self) -> bool {
        let mut sum = 0u8;
        unsafe {
            let ptr = &raw const *self as *const u8;
            for i in 0..self.length as usize {
                sum = sum.wrapping_add(*ptr.add(i));
            }
        }
        sum == 0
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct SdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

impl SdtHeader {
    pub fn validate(&self) -> bool {
        let mut sum = 0u8;
        unsafe {
            let ptr = &raw const *self as *const u8;
            for i in 0..self.length as usize {
                sum = sum.wrapping_add(*ptr.add(i));
            }
        }
        sum == 0
    }
}

pub fn get_xsdp() -> Option<NonNull<Xsdp>> {
    if let Some(res) = RSDP_REQUEST.response() {
        NonNull::new(res.address as *mut Xsdp)
    } else {
        None
    }
}

pub fn is_acpi_available() -> bool {
    get_xsdp().is_some()
}

pub fn find_table_type(table: AcpiTableType) -> Result<Vec<PhysicalAddress>, Error> {
    if let Some(table_addrs) = TABLE_MAP.get(&table) {
        Ok(table_addrs.clone())
    } else {
        Err(Error::TableNotFound)
    }
}

fn parse_xsdt(xsdt_addr: PhysicalAddress) -> HashMap<AcpiTableType, Vec<PhysicalAddress>> {
    let mut tables = HashMap::new();
    logln!("[ACPI] Parsing XSDT at address {:?}", xsdt_addr);
    let xsdt_header: &SdtHeader =
        unsafe { (xsdt_addr.into_hhdm_ptr::<SdtHeader>()).as_ref().unwrap() };
    logln!("[ACPI] XSDT header located.");
    if !xsdt_header.validate() {
        panic!(
            "[ACPI] XSDT checksum validation failed. This machine's firmware is invalid or has \
             been corrupted."
        );
    }
    // Note: HHDM pointers are extremely unsafe as they live entirely outside of Rust's memory model
    // Try to keep their use read-only and never use the HHDM to access data that can be accessed
    // through proper memory mappings. ACPI tables and Device Tree Nodes are an acceptable use case
    // because they are only ever read from.
    let xsdt_data_ptr = unsafe {
        NonNull::new_unchecked((xsdt_addr + size_of::<SdtHeader>()).into_hhdm_mut::<u64>())
    };
    let data_length = xsdt_header.length as usize - size_of::<SdtHeader>();
    let num_entries = data_length / size_of::<u64>();
    logln!(
        "[ACPI] XSDT data physical address: {:?}, number of entries: {}",
        xsdt_data_ptr,
        num_entries
    );
    let mut table_addrs = Vec::<PhysicalAddress>::with_capacity(num_entries);
    for i in 0..num_entries {
        let entry_addr = unsafe { xsdt_data_ptr.as_ptr().add(i).read_unaligned() };
        table_addrs.push(PhysicalAddress::from(entry_addr));
    }

    for table_addr in &table_addrs {
        if let Some(signature) = get_table_signature(*table_addr) {
            if let Ok(table_type) = AcpiTableType::try_from(signature) {
                tables.entry(table_type).or_insert_with(Vec::new).push(*table_addr);
            } else {
                logln!(
                    "[ACPI] Warning: Unrecognized ACPI table with signature {:?} at address {:?}",
                    (unsafe { String::from_utf8_unchecked(signature.to_vec()) }),
                    table_addr
                );
            }
        }
    }
    logln!("[ACPI] Finished parsing XSDT. Found {} tables.", (tables.len()));
    tables
}

fn get_table_signature(table_addr: PhysicalAddress) -> Option<[u8; 4]> {
    if table_addr.is_null() {
        return None;
    } else {
        let header = unsafe { (table_addr.into_hhdm_ptr::<SdtHeader>()).read() };
        Some(header.signature)
    }
}
