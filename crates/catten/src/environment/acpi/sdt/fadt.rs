use super::super::SdtHeader;
use super::*;
use crate::memory::PhysicalAddress;

#[repr(C, packed)]
pub struct Fadt {
    header:               SdtHeader,
    firmware_ctrl:        u32,
    dsdt:                 u32,
    // reserved field used in ACPI 1.0; no longer in use, for compatibility only
    reserved:             u8,
    preferred_pm_profile: u8,
    sci_int:              u16,
    smi_cmd_port:         u32,
    acpi_enable:          u8,
    acpi_disable:         u8,
    s4bios_req:           u8,
    pstate_control:       u8,
    pm1a_evt_blk:         u32,
    pm1b_evt_blk:         u32,
    pm1a_cnt_blk:         u32,
    pm1b_cnt_blk:         u32,
    pm2_cnt_blk:          u32,
    pm_tmr_blk:           u32,
    gpe0_blk:             u32,
    gpe1_blk:             u32,
    pm1_evt_len:          u8,
    pm1_cnt_len:          u8,
    pm2_cnt_len:          u8,
    pm_tmr_len:           u8,
    gpe0_len:             u8,
    gpe1_len:             u8,
    gpe1_base:            u8,
    cstate_ctrl:          u8,
    worst_c2_latency:     u16,
    worst_c3_latency:     u16,
    flush_size:           u16,
    flush_stride:         u16,
    duty_offset:          u8,
    duty_width:           u8,
    day_alarm:            u8,
    month_alarm:          u8,
    century:              u8,
    // reserved in ACPI 1.0; used since ACPI 2.0+
    boot_arch_flags:      u16,
    reserved2:            u8,
    flags:                u32,
    reset_reg:            GenericAddressStructure,
    reset_value:          u8,
    reserved3:            [u8; 3],
    // 64-bit pointers — available on ACPI 2.0+
    x_firmware_ctrl:      u64,
    x_dsdt:               u64,
    x_pm1a_evt_blk:       GenericAddressStructure,
    x_pm1b_evt_blk:       GenericAddressStructure,
    x_pm1a_cnt_blk:       GenericAddressStructure,
    x_pm1b_cnt_blk:       GenericAddressStructure,
    x_pm2_cnt_blk:        GenericAddressStructure,
    x_pm_tmr_blk:         GenericAddressStructure,
    x_gpe0_blk:           GenericAddressStructure,
    x_gpe1_blk:           GenericAddressStructure,
}

impl Fadt {
    pub fn get_dsdt_paddr(&self) -> PhysicalAddress {
        if self.x_dsdt != 0 {
            PhysicalAddress::from(self.x_dsdt)
        } else if self.dsdt != 0 {
            PhysicalAddress::from(self.dsdt as u64)
        } else {
            panic!("[ACPI] The FADT does not contain a non-null physical address for the DSDT.");
        }
    }
}
