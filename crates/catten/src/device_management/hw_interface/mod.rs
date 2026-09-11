/// The software operating interface for a device or more properly, a device function. This is what
/// devices present to the kernel and what drivers use to interact with the device. Userspace does
/// not ever interact with this directly but can query it for debugging and informational purposes.
pub enum HwDeviceIfce {
    Unknown = 0,
    Unsupported = 1,
    // Generic
    Ns16550Uart,
    Ns16650Uart,
    Ns16750Uart,
    Ns16850Uart,
    Ns16950Uart,
    Ns16550MultiPortUart,
    Ns16650MultiPortUart,
    Ns16750MultiPortUart,
    Ns16850MultiPortUart,
    Ns16950MultiPortUart,
    I2CHostController,
    SpiHostController,
    AhciSataController,
    SdHostController,
    ScsiSasController,
    // PCI Express
    PcieHostBridge,
    PciToPciBridgeNormalDecode,
    PciToPciBridgeSubtractiveDecode,
    PciToIsaBridge,
    NvmExpressController,
    // USB
    Usb4Router,
    XhciUsbHostController,
    EhciUsbHostController,
    UsbHidClass,
    CdcAcmVirtualSerial,
    CdcNcmVirtualEthernet,
    //IPMI
    IpmiKcs,
    // Graphics and Display
    AmdGpu,
    ArmGpu,
    IntelGpu,
    NvidiaGpu,
    UefiGopFramebuffer,
    UsbBulkDisplayClass,
    VirtioGpu,
    // x86-64 platform components
    #[cfg(target_arch = "x86_64")]
    I8042InputController,
    #[cfg(target_arch = "x86_64")]
    IoApic,
    #[cfg(target_arch = "x86_64")]
    SmBusController,
    #[cfg(target_arch = "x86_64")]
    IntelVtdIommu,
    #[cfg(target_arch = "x86_64")]
    AmdViIommu,
    #[cfg(target_arch = "x86_64")]
    HighPrecisionEventTimer,
    // Arm platform components
    #[cfg(target_arch = "aarch64")]
    ArmPl011Uart,
    #[cfg(target_arch = "aarch64")]
    ArmGic,
    #[cfg(target_arch = "aarch64")]
    ArmSmmu,
}

/// This trait is implemented to provide human readable names for device interfaces generally for
/// user queries and logging.
impl core::fmt::Display for HwDeviceIfce {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = match self {
            HwDeviceIfce::Unknown => "Unrecognized Device Interface",
            HwDeviceIfce::Unsupported => "Unsupported Device Interface",
            HwDeviceIfce::Ns16550Uart => "NS16550A compatible UART",
            HwDeviceIfce::Ns16650Uart => "NS16650 compatible UART",
            HwDeviceIfce::Ns16750Uart => "NS16750 compatible UART",
            HwDeviceIfce::Ns16850Uart => "NS16850 compatible UART",
            HwDeviceIfce::Ns16950Uart => "NS16950 compatible UART",
            HwDeviceIfce::Ns16550MultiPortUart => "NS16550A compatible Multi-Port UART",
            HwDeviceIfce::Ns16650MultiPortUart => "NS16650 compatible Multi-Port UART",
            HwDeviceIfce::Ns16750MultiPortUart => "NS16750 compatible Multi-Port UART",
            HwDeviceIfce::Ns16850MultiPortUart => "NS16850 compatible Multi-Port UART",
            HwDeviceIfce::Ns16950MultiPortUart => "NS16950 compatible Multi-Port UART",
            HwDeviceIfce::I2CHostController => "Inter-Integrated Circuit (I2C) Host Controller",
            HwDeviceIfce::SpiHostController => "Serial Peripheral Interface (SPI) Host Controller",
            HwDeviceIfce::AhciSataController => "AHCI SATA Controller",
            HwDeviceIfce::SdHostController => "SD Host Controller",
            HwDeviceIfce::ScsiSasController => "Serial Attached SCSI Controller",
            HwDeviceIfce::PcieHostBridge => "PCI Express Host Bridge",
            HwDeviceIfce::PciToPciBridgeNormalDecode => {
                "PCI (Express) to PCI (Express) Bridge (Normal Decode)"
            }
            HwDeviceIfce::PciToPciBridgeSubtractiveDecode => {
                "PCI (Express) to PCI (Express) Bridge (Subtractive Decode)"
            }
            HwDeviceIfce::PciToIsaBridge => "PCI to Legacy ISA Bus Bridge",
            HwDeviceIfce::NvmExpressController => "Non-Volatile Memory Express (NVMe) Controller",
            HwDeviceIfce::Usb4Router => "USB4 Router",
            HwDeviceIfce::XhciUsbHostController => "xHCI compatible USB Host Controller",
            HwDeviceIfce::EhciUsbHostController => "EHCI compatible USB Host Controller",
            HwDeviceIfce::UsbHidClass => "USB Human Interface Device (HID) Class Device",
            HwDeviceIfce::CdcAcmVirtualSerial => {
                "USB Communications Device Class (CDC) Abstract Control Model (ACM) Serial Device"
            }
            HwDeviceIfce::CdcNcmVirtualEthernet => {
                "USB Communications Device Class (CDC) Network Control Model (NCM) Ethernet Device"
            }
            HwDeviceIfce::IpmiKcs => "IPMI KCS Interface",
            HwDeviceIfce::AmdGpu => "AMD VGA Compatible Device, Model Unknown",
            HwDeviceIfce::ArmGpu => "Arm VGA Compatible Device, Model Unknown",
            HwDeviceIfce::IntelGpu => "Intel VGA Compatible Device, Model Unknown",
            HwDeviceIfce::NvidiaGpu => "Nvidia VGA Compatible Device, Model Unknown",
            HwDeviceIfce::UefiGopFramebuffer => "UEFI Graphics Output Protocol (GOP) Framebuffer",
            HwDeviceIfce::UsbBulkDisplayClass => "USB Bulk Display Class Device",
            HwDeviceIfce::VirtioGpu => "VirtIO GPU",
            #[cfg(target_arch = "x86_64")]
            HwDeviceIfce::I8042InputController => "i8042 (PS/2) Compatible Interface",
            #[cfg(target_arch = "x86_64")]
            HwDeviceIfce::IoApic => {
                "IOAPIC/IOxAPIC (I/O Advanced Programmable Interrupt Controller)"
            }
            #[cfg(target_arch = "x86_64")]
            HwDeviceIfce::SmBusController => "System Management Bus (SMBus) Controller",
            #[cfg(target_arch = "x86_64")]
            HwDeviceIfce::IntelVtdIommu => {
                "Intel VT-d IOMMU (Intel Virtualization Technology for Directed I/O)"
            }
            #[cfg(target_arch = "x86_64")]
            HwDeviceIfce::AmdViIommu => "AMD-V IOMMU (Virtualization Technology for I/O)",
            #[cfg(target_arch = "x86_64")]
            HwDeviceIfce::HighPrecisionEventTimer => "High Precision Event Timer (HPET)",
            #[cfg(target_arch = "aarch64")]
            HwDeviceIfce::ArmPl011Uart => "ARM PrimeCell PL011 UART",
            #[cfg(target_arch = "aarch64")]
            HwDeviceIfce::ArmGic => "ARM GIC (Generic Interrupt Controller)",
            #[cfg(target_arch = "aarch64")]
            HwDeviceIfce::ArmSmmu => "ARM SMMU (System Memory Management Unit)",
        };
        write!(f, "{}", name)
    }
}
