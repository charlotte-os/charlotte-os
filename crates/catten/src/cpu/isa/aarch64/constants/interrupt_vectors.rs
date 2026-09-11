//! # AArch64 Interrupt IDs
//!
//! The GIC identifies interrupts by INTID rather than by a vector table index: 0-15 are software
//! generated (SGIs), 16-31 are private peripheral interrupts (PPIs) and 32 upwards are shared
//! peripheral interrupts (SPIs). The assignments below follow that layout.

use crate::cpu::isa::lp::IntSrcDscr;

/// PPI carrying the EL1 physical timer, per the Server Base System Architecture.
pub const LAPIC_TIMER_VECTOR: IntSrcDscr = 30;
/// SGI used for asynchronous IPIs.
pub const ASYNC_IPI_VECTOR: IntSrcDscr = 0;
/// SGI used for synchronous IPIs.
pub const SYNC_IPI_VECTOR: IntSrcDscr = 1;
/// The GIC reports INTID 1023 when there is no pending interrupt to acknowledge.
pub const SPURIOUS_INTERRUPT_VECTOR_NUM: IntSrcDscr = 1023;

/// First SPI available for dynamic assignment.
#[unsafe(no_mangle)]
pub static DYN_VEC_START_OFFSET: u32 = 32;
/// Number of shared peripheral interrupts each LP may be assigned.
pub const DYN_VECS_PER_LP: u32 = 220;
