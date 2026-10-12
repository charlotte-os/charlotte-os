use super::*;
use crate::environment::acpi::table_map::AcpiTableHeader;
use crate::memory::PhysicalAddress;

#[repr(C, packed)]
pub struct Fadt {
    header: AcpiTableHeader,
    firmware_ctrl: u32,
    dsdt: u32,
    // reserved field used in ACPI 1.0; no longer in use, for compatibility only
    reserved: u8,
    preferred_pm_profile: u8,
    sci_int: u16,
    smi_cmd_port: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4bios_req: u8,
    pstate_control: u8,
    pm1a_evt_blk: u32,
    pm1b_evt_blk: u32,
    pm1a_cnt_blk: u32,
    pm1b_cnt_blk: u32,
    pm2_cnt_blk: u32,
    pm_tmr_blk: u32,
    gpe0_blk: u32,
    gpe1_blk: u32,
    pm1_evt_len: u8,
    pm1_cnt_len: u8,
    pm2_cnt_len: u8,
    pm_tmr_len: u8,
    gpe0_len: u8,
    gpe1_len: u8,
    gpe1_base: u8,
    cstate_ctrl: u8,
    worst_c2_latency: u16,
    worst_c3_latency: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alarm: u8,
    month_alarm: u8,
    century: u8,
    // reserved in ACPI 1.0; used since ACPI 2.0+
    iapc_boot_arch: u16,
    reserved2: u8,
    flags: u32,
    reset_reg: GenericAddressStructure,
    reset_value: u8,
    arm_boot_arch: u16,
    fadt_minor_version: u8,
    // 64-bit pointers — available on ACPI 2.0+
    x_firmware_ctrl: u64,
    x_dsdt: u64,
    x_pm1a_evt_blk: GenericAddressStructure,
    x_pm1b_evt_blk: GenericAddressStructure,
    x_pm1a_cnt_blk: GenericAddressStructure,
    x_pm1b_cnt_blk: GenericAddressStructure,
    x_pm2_cnt_blk: GenericAddressStructure,
    x_pm_tmr_blk: GenericAddressStructure,
    x_gpe0_blk: GenericAddressStructure,
    x_gpe1_blk: GenericAddressStructure,
}

cfg_select! {
    target_arch = "x86_64" => {
        pub struct IapcBootArchFlags(u16);

        impl IapcBootArchFlags {
            pub fn legacy_devices_present(&self) -> bool {
                (self.0 & 0x1) != 0
            }

            pub fn i8042_may_be_present(&self) -> bool {
                (self.0 & 0x2) != 0
            }

            pub fn vga_not_present(&self) -> bool {
                (self.0 & 0x4) != 0
            }

            pub fn msi_not_supported(&self) -> bool {
                (self.0 & 0x8) != 0
            }

            pub fn pcie_aspm_control_not_available(&self) -> bool {
                (self.0 & 0x10) != 0
            }

            pub fn rtc_not_present(&self) -> bool {
                (self.0 & 0x20) != 0
            }
        }
    }
    target_arch = "aarch64" => {
        pub struct ArmBootArchFlags(u16);

        impl ArmBootArchFlags {
            pub fn psci_compliant(&self) -> bool {
                (self.0 & 0x1) != 0
            }

            pub fn psci_use_hvc(&self) -> bool {
                (self.0 & 0x2) != 0
            }
        }
    }
}

impl Fadt {
    #[cfg(target_arch = "x86_64")]
    pub fn get_iapc_boot_arch_flags(&self) -> IapcBootArchFlags {
        IapcBootArchFlags(self.iapc_boot_arch)
    }

    #[cfg(target_arch = "aarch64")]
    pub fn get_arm_boot_arch_flags(&self) -> ArmBootArchFlags {
        ArmBootArchFlags(self.arm_boot_arch)
    }
}
