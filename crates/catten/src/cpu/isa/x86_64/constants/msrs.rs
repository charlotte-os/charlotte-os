//! # x86-64 Model Specific Registers (MSRs)
//!
//! Make sure to check any necessary CPUID features before using these MSRs as not all of them are
//! architectural.

#[inline(always)]
pub unsafe fn read(msr: u32) -> u64 {
    //! Read from an x86-64 model specific register
    let low: u32;
    let high: u32;

    unsafe {
        core::arch::asm!(
            "rdmsr",
            out("eax") low,
            out("edx") high,
            in("ecx") msr,
            options(nomem, nostack)
        );
    }

    ((high as u64) << 32) | (low as u64)
}

#[inline(always)]
pub unsafe fn write(msr: u32, value: u64) {
    //! Write to an x86-64 model specific register
    let low = value as u32;
    let high = (value >> 32) as u32;

    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("eax") low,
            in("edx") high,
            in("ecx") msr,
            options(nomem, nostack)
        );
    }
}

pub const APIC_BASE_ADDRESS_REG: u32 = 0x1b;
pub mod x2apic {
    //! # x2APIC MSRs
    //! Ref: AMD APM 16.11.1 and Intel SDM Vol.3 12.12.1.2
    pub const ID_REG: u32 = 0x802;
    pub const EOI_REGISTER: u32 = 0x80b;
    pub const LOGICAL_DEST_REG: u32 = 0x80d;
    pub const SPURIOUS_INTERRUPT_VECTOR_REG: u32 = 0x80f;
    pub const INTERRUPT_COMMAND_REGISTER: u32 = 0x830;
    pub const TIMER_LVTR: u32 = 0x832;
    pub const TIMER_INITIAL_COUNT: u32 = 0x838;
    pub const TIMER_CURRENT_COUNT: u32 = 0x839;
    pub const TIMER_DIVIDE_CONFIGURATION: u32 = 0x83e;
}

/// # TSC_AUX MSR
pub const TSC_AUX: u32 = 0xc000_0103;
