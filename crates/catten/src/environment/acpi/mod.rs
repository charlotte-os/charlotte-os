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

pub mod aml;
pub mod sdt;
pub mod table_map;
