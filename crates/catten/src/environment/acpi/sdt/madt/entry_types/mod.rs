mod interrupt_source_override;
mod ioapic;
mod local_x2apic_nmi;
mod nmi_source;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MadtEntryType {
    /* The Local APIC and all associated entry types are not used because this kernel requires
     * x86-64 based machines to support x2APIC mode. */
    _LocalApic = 0x0,
    IoApic = 0x1,
    InterruptSourceOverride = 0x2,
    NmiSource = 0x3,
    _LocalApicNmi = 0x4,
    _LocalApicAddressOverride = 0x5,
    /* The following three types are specific to IA-64, an ISA this kernel will never support
     * due to it being EOL. */
    _IoSapic = 0x6,
    _LocalSapic = 0x7,
    _PlatformInterruptSource = 0x8,
    ProcessorLocalX2Apic = 0x9,
    LocalX2ApicNmi = 0xa,
    /* The Aarch64 specific entries are only to be used on Aarch64 platforms. */
    GicCpuInterface = 0xb,
    GicDistributor = 0xc,
    GicMsiFrame = 0xd,
    GicRedistributor = 0xe,
    GicInterruptTranslationService = 0xf,
    /* The Limine MP feature is always used on all UEFI/ACPI platforms so this entry type is never used. */
    _MultiprocessorWakeup = 0x10,
    /* The following seven entry types are never used by this kernel. */
    _CoreProgrammableInterruptController = 0x11,
    _LegacyIoProgrammableInterruptController = 0x12,
    _HyperTransportProgrammableInterruptController = 0x13,
    _ExtendIoProgrammableInterruptController = 0x14,
    _MsiProgrammableInterruptController = 0x15,
    _BridgeIoProgrammableInterruptController = 0x16,
    _LowPinCountProgrammableInterruptController = 0x17,
    /* The RISC-V specific entries are only to be used on RISC-V platforms. */
    RiscVHartLocalInterruptController = 0x18,
    RiscVIncomingMsiController = 0x19,
    RiscVAdvancedPlatformLevelInterruptController = 0x1a,
    RiscVPlatformLevelInterruptController = 0x1b,
}
