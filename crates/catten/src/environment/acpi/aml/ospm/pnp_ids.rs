//! # Plug and Play BIOS Device IDs
//!
//! Plug and Play BIOS Device ID constants for use with ACPI

use core::ffi::CStr;

/* Legacy PS/2 Devices */
pub const PS2_KEYBOARD: &CStr = c"PNP0303";
pub const PS2_MOUSE: &CStr = c"PNP0F03";

/* PCI and PCI Express */
pub const PCI_EXPRESS_ROOT_COMPLEX: &CStr = c"PNP0A08";
pub const PCI_ROOT_COMPLEX: &CStr = c"PNP0A03";
