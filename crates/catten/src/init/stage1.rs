use crate::cpu::isa::lp::ops::{get_lp_id, mask_interrupts, unmask_interrupts};
use crate::cpu::scheduler::{abort, spawn_thread};
use crate::device_management::drivers::busses::pci_express::topology::PCIE_TOPOLOGY;
use crate::environment::acpi::aml::ospm::osc::pcie::negotiate_pci_osc;
use crate::logln;
use crate::memory::KERNEL_ASID;

pub fn create_initial_threads() {
    mask_interrupts!();
    let thread_id = spawn_thread(KERNEL_ASID, initialize_platform);
    logln!("Platform initialization thread spawned with ID = {thread_id}.");
    unmask_interrupts!();
    logln!("Submitted all initial kernel threads.");
}

/// Firmware initialization precedes discovery so drivers can safely evaluate AML.
extern "C" fn initialize_platform() {
    #[cfg(target_arch = "x86_64")]
    print_ioapic_info();
    #[cfg(feature = "acpi")]
    {
        crate::environment::acpi::aml::ospm::init::initialize_acpi()
            .unwrap_or_else(|error| panic!("ACPI initialization failed: {error}"));
        logln!("LP {}: ACPI initialization complete.", (get_lp_id()));
        unsafe {
            use crate::environment::acpi::aml::ospm::pnp_ids;

            uacpi_wrapper::uacpi_find_devices(
                pnp_ids::PS2_KEYBOARD.as_ptr(),
                Some(ps2_kb_status_print),
                core::ptr::null_mut(),
            );
        }
    }
    logln!("{}", (*PCIE_TOPOLOGY));
    negotiate_pci_osc();
    logln!("Platform initialization complete.");
    abort();
}

#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub extern "C" fn print_ioapic_info() {
    use crate::device_management::drivers::platform_devices::wired_interrupt_controller::ioapic::IOAPIC_LIST;

    logln!("LP {}: Printing IOAPIC information...", (get_lp_id()));
    let ioapic_list = IOAPIC_LIST.read();
    ioapic_list.iter().for_each(|(id, desc)| {
        logln!("Enumerated IOAPIC with ID = {:?}: {:?}", id, (desc.lock()));
    });
}

#[unsafe(no_mangle)]
#[cfg(feature = "acpi")]
extern "C" fn ps2_kb_status_print(
    _: *mut core::ffi::c_void,
    node: *mut uacpi_wrapper::uacpi_namespace_node,
    _: u32,
) -> uacpi_wrapper::uacpi_iteration_decision {
    use uacpi_wrapper::UACPI_STATUS_OK;

    let mut kb_res: *mut uacpi_wrapper::uacpi_resources = core::ptr::null_mut();

    let ret = unsafe {
        use uacpi_wrapper::uacpi_get_current_resources;
        uacpi_get_current_resources(node, &mut kb_res)
    };
    if core::hint::unlikely(ret != UACPI_STATUS_OK) {
        use core::ffi::CStr;

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
        logln!("Successfully retrieved PS/2 keyboard resources from ACPI.");
    }
    uacpi_wrapper::UACPI_ITERATION_DECISION_CONTINUE
}
