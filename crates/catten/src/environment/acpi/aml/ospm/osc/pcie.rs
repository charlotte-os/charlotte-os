use alloc::ffi::CString;

use crate::environment::acpi::aml::ospm::pnp_ids;
use crate::environment::acpi::sdt::fadt;
#[cfg(target_arch = "x86_64")]
use crate::environment::acpi::table_map::find_table_type;
use crate::logln;
#[cfg(target_arch = "x86_64")]
use crate::memory::physical::PhysicalAddressIfce;

pub fn pcie_claim_osc() {
    #[cfg(target_arch = "x86_64")]
    {
        let boot_flags = unsafe {
            use crate::environment::acpi::table_map::AcpiTableType;

            find_table_type(AcpiTableType::FADT).expect("FADT table not found")[0]
                .into_hhdm_ptr::<fadt::Fadt>()
                .read()
                .get_iapc_boot_arch_flags()
        };
        if boot_flags.msi_not_supported() {
            panic!(
                "PCI Express Message Signaled Interrupt support is required but not available on \
                 this system"
            );
        }
    }
    let status = unsafe {
        uacpi_wrapper::uacpi_find_devices(
            pnp_ids::PCI_EXPRESS_ROOT_COMPLEX.as_ptr(),
            Some(negotiate_osc),
            core::ptr::null_mut(),
        )
    };
    if status != 0 {
        let error_message = unsafe { uacpi_wrapper::uacpi_status_to_string(status) };
        logln!(
            "[ACPI][OSPM] uACPI error occurred while finding PCI Express Root Complex devices: \
             {:?}",
            unsafe { CString::from_raw(error_message as *mut i8) }
        );
    }
}

extern "C" fn negotiate_osc(
    _: *mut core::ffi::c_void,
    node: *mut uacpi_wrapper::uacpi_namespace_node,
    _: u32,
) -> uacpi_wrapper::uacpi_iteration_decision {
    todo!("Implement PCI Express OSC negotiation");
}

fn invoke_pcie_osc(node: *mut uacpi_wrapper::uacpi_namespace_node) -> uacpi_wrapper::uacpi_status {
    todo!()
}
