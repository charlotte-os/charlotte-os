#![no_std]
#![no_main]
#![feature(extend_one)]
#![feature(iter_advance_by)]
#![feature(likely_unlikely)]
#![feature(step_trait)]
#![allow(static_mut_refs)]
#![allow(named_asm_labels)]

//! # Catten
//!
//! Catten is an operating system kernel developed as a component of CharlotteOS, an
//! experimental modern operating system.It is responsible for initializing the hardware,
//! providing common abstractions for all hardware resources, and managing the execution of
//! user-space applications and the environment in which they run. It is a crucial part of the
//! operating system, as it provides the foundation on which the rest of the system is built and it
//! touches every hardware and software component of the system on which it is used. While it is
//! developed as a component of CharlotteOS, it is designed to be modular and flexible, and thus
//! useful in other operating systems, embedded firmware, and other types of software systems
//! as well.

extern crate alloc;

pub mod cpu;
pub mod device_management;
pub mod environment;
pub mod init;
pub mod klib;
pub mod log;
pub mod memory;
pub mod panic;
pub mod power_management;
pub mod self_test;
pub mod timers;

use core::hint::unreachable_unchecked;

use limine::mp::MpInfo;

use crate::cpu::isa::interface::interrupts::LocalIntCtlrIfce;
use crate::cpu::isa::interface::system_info::CpuInfoIfce;
use crate::cpu::isa::interrupts::LocalIntCtlr;
use crate::cpu::isa::lp::ops::get_lp_id;
use crate::cpu::isa::system_info::CpuInfo;
use crate::cpu::isa::timers::print_timer_info;
use crate::cpu::multiprocessor::startup::{assign_id, start_secondary_lps};
use crate::cpu::scheduler::yield_lp;

const KERNEL_VERSION: (u64, u64, u64) = (0, 10, 0);
/// This is the bootstrap processor's entry point into the kernel. The `bsp_main` function is
/// called by the bootloader after setting up the environment. It is made C ABI compatible so
/// that it can be called by Limine or any other Limine Boot Protocol compliant bootloader.
#[unsafe(no_mangle)]
pub extern "C" fn bsp_main() -> ! {
    early_logln!(
        "Catten Kernel Version {}.{}.{}",
        (KERNEL_VERSION.0),
        (KERNEL_VERSION.1),
        (KERNEL_VERSION.2)
    );
    early_logln!("========================================================================");
    early_logln!("Initializing the system using the bootstrap processor...");
    unsafe {
        assign_id();
    }
    early_logln!("BSP assigned ID 0.");
    init::stage0::bsp_init();
    logln!("System initialized.");
    logln!("Starting secondary LPs...");
    start_secondary_lps().expect("Failed to start secondary LPs");
    self_test::run_self_tests();
    logln!("System Information:");
    logln!("CPU Vendor: {}", (CpuInfo::get_vendor()));
    logln!("CPU Model: {}", (CpuInfo::get_model()));
    logln!("Physical Address bits implemented: {}", (CpuInfo::get_paddr_sig_bits()));
    logln!("Virtual Address bits implemented: {}", (CpuInfo::get_vaddr_sig_bits()));
    print_timer_info();
    cfg_select! {
        feature = "acpi" => {
            environment::acpi::table_map::print_table_map();
        }
    }
    init::stage1::create_initial_threads();
    logln!(
        "LP {}: Bootstrapping complete. Yielding the processor to the scheduler.",
        (get_lp_id())
    );
    LocalIntCtlr::init_lp();
    logln!(
        "LP {}: Initialized local interrupt controller. Yielding the processor to the scheduler.",
        (get_lp_id())
    );
    yield_lp();
    /* We've switched into thread context and never come back */
    unsafe { unreachable_unchecked() }
}
/// This is the application processors' entry point into the kernel. The `ap_main` function is
/// called by each application processor upon entering the kernel. It initializes the processor and
/// then hands it off to the scheduler. It is made C ABI compatible so that it can work with the
/// Limine Boot Protocol MP feature. Other boot protocols may require alternate implementations of
/// `ap_main`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ap_main(_cpuinfo: &MpInfo) -> ! {
    unsafe {
        assign_id();
    }
    init::stage0::ap_init();
    let lp_id = get_lp_id();
    logln!("LP {lp_id}: Bootstrapping complete.");
    logln!("LP {lp_id}: Starting local interrupt controller initialization.");
    LocalIntCtlr::init_lp();
    logln!(
        "LP {lp_id}: Initialized local interrupt controller. Yielding the processor to the \
         scheduler."
    );
    yield_lp();
    /* We've switched into thread context and never come back */
    unsafe { unreachable_unchecked() }
}
