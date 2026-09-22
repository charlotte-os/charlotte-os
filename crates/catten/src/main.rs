#![no_std]
#![no_main]
#![feature(cstr_bytes)]
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

use alloc::string::ToString;
use core::ffi::CStr;
use core::hint::unreachable_unchecked;

use limine::mp::MpInfo;
use spin::{Barrier, LazyLock};
use uacpi_wrapper::{UACPI_STATUS_OK, uacpi_get_current_resources};

use crate::cpu::isa::interface::interrupts::LocalIntCtlrIfce;
use crate::cpu::isa::interface::system_info::CpuInfoIfce;
use crate::cpu::isa::interrupts::LocalIntCtlr;
use crate::cpu::isa::lp::ops::get_lp_id;
use crate::cpu::isa::system_info::CpuInfo;
use crate::cpu::isa::timers::print_timer_info;
use crate::cpu::multiprocessor::get_lp_count;
use crate::cpu::multiprocessor::startup::{assign_id, start_secondary_lps};
use crate::cpu::scheduler::system_scheduler::SYSTEM_SCHEDULER;
use crate::cpu::scheduler::{spawn_thread_on_lp, yield_lp};
use crate::device_management::drivers::busses::pci_express::topology::PCIE_TOPOLOGY;
#[cfg(target_arch = "x86_64")]
use crate::device_management::drivers::platform_devices::wired_interrupt_controller::ioapic::IOAPIC_LIST;
use crate::memory::KERNEL_ASID;

const KERNEL_VERSION: (u64, u64, u64) = (0, 10, 0);
static INIT_BARRIER: LazyLock<Barrier> = LazyLock::new(|| Barrier::new(get_lp_count() as usize));
static YIELD_BARRIER: LazyLock<Barrier> = LazyLock::new(|| Barrier::new(get_lp_count() as usize));
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
    init::bsp_init();
    logln!("System initialized.");
    logln!("Starting secondary LPs...");
    start_secondary_lps().expect("Failed to start secondary LPs");
    INIT_BARRIER.wait();
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
    mask_interrupts!();
    let thread_id = spawn_thread_on_lp(KERNEL_ASID, initialize_platform, 0);
    logln!("Platform initialization thread spawned with ID = {thread_id}.");
    unmask_interrupts!();
    logln!("Submitted all initial kernel threads.");
    logln!(
        "LP {}: Bootstrapping complete. Yielding the processor to the scheduler.",
        (get_lp_id())
    );
    YIELD_BARRIER.wait();
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
    init::ap_init();
    INIT_BARRIER.wait();
    let lp_id = get_lp_id();
    logln!("LP {lp_id}: Bootstrapping complete.");
    YIELD_BARRIER.wait();
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

#[unsafe(no_mangle)]
pub extern "C" fn print_pcie_topology() {
    logln!("LP {}: Probing PCIe topology...", (get_lp_id()));
    let topology = &*PCIE_TOPOLOGY;
    logln!("LP {}: PCIe Topology:\r\n{}", (get_lp_id()), (topology.lock().to_string()));
}

#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub extern "C" fn print_ioapic_info() {
    logln!("LP {}: Printing IOAPIC information...", (get_lp_id()));
    let ioapic_list = IOAPIC_LIST.read();
    ioapic_list.iter().for_each(|(id, desc)| {
        logln!("Enumerated IOAPIC with ID = {:?}: {:?}", id, (desc.lock()));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn test_fn() {
    let thread_id = SYSTEM_SCHEDULER.read().get_lp_scheduler().lock().get_tid().unwrap();
    let lp_id = get_lp_id();
    loop {
        logln!("Logging from thread {thread_id} on LP {lp_id}!");
    }
}

#[unsafe(no_mangle)]
#[cfg(feature = "acpi")]
extern "C" fn ps2_kb_status_print(
    _: *mut core::ffi::c_void,
    node: *mut uacpi_wrapper::uacpi_namespace_node,
    _: u32,
) -> uacpi_wrapper::uacpi_iteration_decision {
    let mut kb_res: *mut uacpi_wrapper::uacpi_resources = core::ptr::null_mut();

    let ret = unsafe { uacpi_get_current_resources(node, &mut kb_res) };
    if core::hint::unlikely(ret != UACPI_STATUS_OK) {
        logln!(
            "unable to retrieve PS/2 keyboard resources: {}",
            (unsafe {
                CStr::from_ptr(uacpi_wrapper::uacpi_status_to_string(ret))
                    .to_str()
                    .unwrap_or("<invalid UTF-8>")
            })
        );
        return uacpi_wrapper::UACPI_ITERATION_DECISION_NEXT_PEER;
    } else {
        logln!("Successfully retrieved PS/2 keyboard resources.");
    }
    uacpi_wrapper::UACPI_ITERATION_DECISION_CONTINUE
}

/// Firmware initialization precedes discovery so drivers can safely evaluate AML.
extern "C" fn initialize_platform() {
    #[cfg(target_arch = "x86_64")]
    print_ioapic_info();
    #[cfg(feature = "acpi")]
    {
        crate::environment::acpi::aml::init::initialize_acpi()
            .unwrap_or_else(|error| panic!("ACPI initialization failed: {error}"));
        logln!("LP {}: ACPI initialization complete.", (get_lp_id()));
        unsafe {
            uacpi_wrapper::uacpi_find_devices(
                c"PNP0303".as_ptr(),
                Some(ps2_kb_status_print),
                core::ptr::null_mut(),
            );
        }
    }
    print_pcie_topology();
    logln!("Platform initialization complete.");
}
