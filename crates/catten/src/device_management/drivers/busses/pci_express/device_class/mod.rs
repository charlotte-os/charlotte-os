mod constants;

use core::fmt::Display;

pub use constants::*;

use crate::device_management::hw_interface::HwDeviceIfce;

/// The PCI identifier is the collection of fields that identify a PCI device and are used to
/// determine what driver should be used to operate the device. It is derived from the PCI
/// configuration space header and consists of the vendor ID, device ID, class code, subclass, and
/// programming interface fields.
///
/// The source for all IDs and codes used in this module and all its submodules is `https://pci-ids.ucw.cz/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PciIdentifier {
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
}

impl Display for PciIdentifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "| {}[0x{:04x}]:0x{:04x} | {}[class 0x{:02x}, subclass 0x{:02x}, prog IF 0x{:02x}] |",
            constants::vendor_id::pci_vendor_id_to_str(self.vendor_id),
            self.vendor_id,
            self.device_id,
            Into::<HwDeviceIfce>::into(*self),
            self.class_code,
            self.subclass,
            self.prog_if,
        )
    }
}

impl Into<HwDeviceIfce> for PciIdentifier {
    fn into(self) -> HwDeviceIfce {
        match (self.vendor_id, self.device_id, (self.class_code, self.subclass, self.prog_if)) {
            /* AMD */
            (vendor_id::AMD, _, device_class::VGA_COMPATIBLE) => HwDeviceIfce::AmdGpu,
            #[cfg(target_arch = "x86_64")]
            (vendor_id::AMD, _, device_class::IOMMU) => HwDeviceIfce::AmdViIommu,

            /* ARM */
            #[cfg(target_arch = "aarch64")]
            (vendor_id::ARM, _, device_class::VGA_COMPATIBLE) => HwDeviceIfce::ArmGpu,

            /* Intel */
            (vendor_id::INTEL, _, device_class::VGA_COMPATIBLE) => HwDeviceIfce::IntelGpu,
            #[cfg(target_arch = "x86_64")]
            (vendor_id::INTEL, _, device_class::IOMMU) => HwDeviceIfce::IntelVtdIommu,

            /* Nvidia */
            (vendor_id::NVIDIA, _, device_class::VGA_COMPATIBLE) => HwDeviceIfce::NvidiaGpu,

            /* virtio */
            // This is technically virtio-vga but we don't use the legacy VGA interface just the one
            // it shares with virtio-gpu so we map it to virtio-gpu.
            #[cfg(feature = "virtio_gpu")]
            (
                vendor_id::REDHAT_VIRTIO | vendor_id::REDHAT,
                _,
                device_class::VGA_COMPATIBLE | device_class::OTHER_DISPLAY_CONTROLLER,
            ) => HwDeviceIfce::VirtioGpu,

            /* Generic Device Class Interfaces

            These purposely bind less tightly than vendor specific matches since there could be devices that use the
            same class codes but require device specific drivers or special handling in the same driver.
            */
            (_, _, device_class::AHCI_SATA_CONTROLLER) => HwDeviceIfce::AhciSataController,
            (_, _, device_class::SCSI_SAS_CONTROLLER) => HwDeviceIfce::ScsiSasController,
            (_, _, device_class::NVME_CONTROLLER) => HwDeviceIfce::NvmExpressController,
            (_, _, device_class::HOST_BRIDGE) => HwDeviceIfce::PcieHostBridge,
            (_, _, device_class::PCI_TO_PCI_BRIDGE) => HwDeviceIfce::PciToPciBridgeNormalDecode,
            (_, _, device_class::PCI_TO_PCI_BRIDGE_SUB_DEC) => {
                HwDeviceIfce::PciToPciBridgeSubtractiveDecode
            }
            (_, _, device_class::PCI_TO_ISA_BRIDGE) => HwDeviceIfce::PciToIsaBridge,
            (_, _, device_class::NS16550) => HwDeviceIfce::Ns16550Uart,
            (_, _, device_class::NS16650) => HwDeviceIfce::Ns16650Uart,
            (_, _, device_class::NS16750) => HwDeviceIfce::Ns16750Uart,
            (_, _, device_class::NS16850) => HwDeviceIfce::Ns16850Uart,
            (_, _, device_class::NS16950) => HwDeviceIfce::Ns16950Uart,
            (_, _, device_class::NS16550_MULTI_PORT) => HwDeviceIfce::Ns16550MultiPortUart,
            (_, _, device_class::NS16650_MULTI_PORT) => HwDeviceIfce::Ns16650MultiPortUart,
            (_, _, device_class::NS16750_MULTI_PORT) => HwDeviceIfce::Ns16750MultiPortUart,
            (_, _, device_class::NS16850_MULTI_PORT) => HwDeviceIfce::Ns16850MultiPortUart,
            (_, _, device_class::NS16950_MULTI_PORT) => HwDeviceIfce::Ns16950MultiPortUart,
            #[cfg(target_arch = "x86_64")]
            (_, _, device_class::IOAPIC | device_class::IOXAPIC) => HwDeviceIfce::IoApic,
            #[cfg(target_arch = "x86_64")]
            (_, _, device_class::HPET) => HwDeviceIfce::HighPrecisionEventTimer,
            (_, _, device_class::SD_HOST_CONTROLLER) => HwDeviceIfce::SdHostController,
            (_, _, device_class::USB_EHCI) => HwDeviceIfce::EhciUsbHostController,
            (_, _, device_class::USB_XHCI) => HwDeviceIfce::XhciUsbHostController,
            (_, _, device_class::USB4_ROUTER) => HwDeviceIfce::Usb4Router,
            (_, _, device_class::SMBUS_CONTROLLER) => HwDeviceIfce::SmBusController,
            (_, _, device_class::IPMI_KCS) => HwDeviceIfce::IpmiKcs,

            /* Unrecognized Devices */
            (_, _, (_, _, _)) => HwDeviceIfce::Unknown,
        }
    }
}
