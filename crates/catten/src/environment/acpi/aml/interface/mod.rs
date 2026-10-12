use crate::environment::acpi::sdt::fadt;
#[cfg(target_arch = "x86_64")]
use crate::environment::acpi::table_map::find_table_type;
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
                "PCI ExpressMessage Signaled Interrupt support is required but not available on \
                 this system"
            );
        }
        if boot_flags.pcie_aspm_control_not_available() {
            panic!(
                "Operating System controlled PCI Express Active State Power Management is \
                 required but not available on this system"
            );
        }
    }
}
