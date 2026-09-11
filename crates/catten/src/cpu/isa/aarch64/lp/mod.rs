//! # Logical Processor Control Interface for AArch64
pub mod ops;
pub mod thread_context;

/// Logical processor ID, derived from `MPIDR_EL1`'s affinity fields.
pub type LpId = u32;
pub type CoreId = u32;

/// Identifies a wired interrupt controller (a GIC distributor instance).
pub type WiredIntCtlrId = u8;
/// A GIC distributor's input line number.
pub type WiredIntCtlrSrcNum = u8;
/// Identifies an interrupt source as seen by a logical processor: a GIC INTID.
pub type IntSrcDscr = u32;
