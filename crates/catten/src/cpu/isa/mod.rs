//! # Instruction Set Architecture (ISA) Interface
//!
//! This module provides a set of common interfaces used to access the ISA specific
//! functionality needed by the kernel:
//! - [`Constants`](constants): ISA specific constant values
//! - [`Initialization`](init): ISA specific system initialization and deinitialization
//! - [`Interrupts`](interrupts): wrappers over ISA specific interrupt management structures
//! - [`Input/Output`](io): wrappers over MMIO and Port IO
//! - [`Logical Processor Control`](lp): logical processor operating state control
//! - [`Memory`](memory): wrappers over ISA specific memory management structures
//! - [`System Information`](system_info): ISA specific system information
//! - [`Timers`](timers): ISA specific timer management structures

// Contains components common to some though not necessarily all ISAs
mod common;
pub mod interface;

cfg_select! {
    target_arch = "aarch64" => {
        mod aarch64;
        pub use aarch64::*;
    }
    target_arch = "riscv64" => {
        mod riscv64;
        pub use riscv64::*;
    }
    target_arch = "x86_64" => {
        mod x86_64;
        pub use x86_64::*;
    }
}
