//! # RISC-V Interrupt Source Numbers
//!
//! RISC-V has no interrupt vector table to index into: local interrupts are identified by the
//! exception code in `scause` and external ones by their PLIC/APLIC source number. The names below
//! exist so that architecture-neutral callers keep compiling; the numbering that makes them
//! meaningful arrives with the interrupt controller driver.

/// Supervisor timer interrupt, `scause` exception code 5.
pub const LAPIC_TIMER_VECTOR: u8 = 5;
/// Supervisor software interrupt, `scause` exception code 1, used for asynchronous IPIs.
pub const ASYNC_IPI_VECTOR: u8 = 1;
/// Synchronous IPIs share the supervisor software interrupt and are told apart by a mailbox.
pub const SYNC_IPI_VECTOR: u8 = 1;
/// RISC-V has no spurious interrupt vector; the PLIC reports source 0 for "no interrupt".
pub const SPURIOUS_INTERRUPT_VECTOR_NUM: u8 = 0;

/// First external interrupt source number available for dynamic assignment.
#[unsafe(no_mangle)]
pub static DYN_VEC_START_OFFSET: u8 = 1;
/// Number of external interrupt sources each hart may be assigned.
pub const DYN_VECS_PER_LP: u8 = 127;
