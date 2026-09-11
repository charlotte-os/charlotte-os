//! # RISC-V Hart (Logical Processor) Control

pub mod ops;
pub mod thread_context;

/// Hart ID, as reported by `mhartid` and passed down to S-mode by the boot protocol.
pub type LpId = u32;
pub type CoreId = u32;

/// Identifies a wired interrupt controller (a PLIC or APLIC instance).
pub type WiredIntCtlrId = u8;
/// A wired interrupt controller's input line number.
pub type WiredIntCtlrSrcNum = u8;
/// Identifies an interrupt source as seen by a hart.
pub type IntSrcDscr = u8;
