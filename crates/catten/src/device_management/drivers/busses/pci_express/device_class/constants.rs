//! Named PCI IDs and class codes for use in driver matching.
//!
//! Only IDs that the kernel needs to refer to by name are defined here. Names for every known
//! vendor, device and class are looked up from the PCI ID Repository through [`super::pci_ids`].

pub mod vendor_id {
    pub type PciVendorId = u16;

    pub const VENDOR_ID_UNKNOWN: PciVendorId = 0xffff;
    pub const VENDOR_ID_INVALID: PciVendorId = 0x0000;

    /* CPU, Chipset and Platform Vendors */
    pub const INTEL: PciVendorId = 0x8086;
    pub const AMD: PciVendorId = 0x1022;
    // Used for AMD (formerly ATI) GPUs and their companion devices such as HDMI audio.
    pub const AMD_ATI: PciVendorId = 0x1002;
    pub const ARM: PciVendorId = 0x13b5;
    pub const QUALCOMM: PciVendorId = 0x17cb;
    pub const APPLE: PciVendorId = 0x106b;
    pub const AMPERE: PciVendorId = 0x1def;
    pub const CAVIUM: PciVendorId = 0x177d;
    pub const HUAWEI: PciVendorId = 0x19e5;
    pub const LOONGSON: PciVendorId = 0x0014;
    pub const PHYTIUM: PciVendorId = 0x1db7;
    pub const ZHAOXIN: PciVendorId = 0x1d17;
    pub const VIA: PciVendorId = 0x1106;
    pub const FREESCALE: PciVendorId = 0x1957;

    /* GPU and Display Vendors */
    pub const NVIDIA: PciVendorId = 0x10de;
    pub const MATROX: PciVendorId = 0x102b;
    pub const ASPEED: PciVendorId = 0x1a03;
    pub const S3_GRAPHICS: PciVendorId = 0x5333;
    pub const CIRRUS_LOGIC: PciVendorId = 0x1013;
    pub const MOORE_THREADS: PciVendorId = 0x1ed5;

    /* Virtual Machine and Cloud Vendors */
    // Not in the PCI ID Repository; used by QEMU for its standard VGA and emulated test devices.
    pub const QEMU: PciVendorId = 0x1234;
    // Red Hat, Inc. (primarily used for virtio devices)
    pub const REDHAT_VIRTIO: PciVendorId = 0x1af4;
    // Red Hat, Inc. (primarily used for other QEMU emulated devices)
    pub const REDHAT: PciVendorId = 0x1b36;
    pub const VMWARE: PciVendorId = 0x15ad;
    pub const MICROSOFT: PciVendorId = 0x1414;
    pub const XEN: PciVendorId = 0x5853;
    pub const VIRTUALBOX: PciVendorId = 0x80ee;
    pub const PARALLELS: PciVendorId = 0x1ab8;
    pub const AMAZON: PciVendorId = 0x1d0f;
    pub const GOOGLE: PciVendorId = 0x1ae0;

    /* Networking Vendors */
    pub const BROADCOM: PciVendorId = 0x14e4;
    pub const REALTEK: PciVendorId = 0x10ec;
    pub const QUALCOMM_ATHEROS: PciVendorId = 0x168c;
    pub const MEDIATEK: PciVendorId = 0x14c3;
    pub const MELLANOX: PciVendorId = 0x15b3;
    pub const AQUANTIA: PciVendorId = 0x1d6a;
    pub const MARVELL: PciVendorId = 0x11ab;

    /* Storage Vendors */
    pub const BROADCOM_LSI: PciVendorId = 0x1000;
    pub const MARVELL_EXT: PciVendorId = 0x1b4b;
    pub const SAMSUNG: PciVendorId = 0x144d;
    pub const SK_HYNIX: PciVendorId = 0x1c5c;
    pub const MICRON: PciVendorId = 0x1344;
    pub const SANDISK: PciVendorId = 0x15b7;
    pub const KINGSTON: PciVendorId = 0x2646;
    pub const PHISON: PciVendorId = 0x1987;
    pub const SILICON_MOTION: PciVendorId = 0x126f;
    pub const TOSHIBA: PciVendorId = 0x1179;
    pub const KIOXIA: PciVendorId = 0x1e0f;
    pub const SEAGATE: PciVendorId = 0x1bb1;
    pub const ADATA: PciVendorId = 0x1cc1;
    pub const MAXIO: PciVendorId = 0x1e4b;
    pub const JMICRON: PciVendorId = 0x197b;
    pub const MICROSEMI: PciVendorId = 0x111d;

    /* USB, Bridge and Peripheral Controller Vendors */
    pub const ASMEDIA: PciVendorId = 0x1b21;
    pub const RENESAS: PciVendorId = 0x1912;
    pub const FRESCO_LOGIC: PciVendorId = 0x1b73;
    pub const ETRON: PciVendorId = 0x1b6f;
    pub const TEXAS_INSTRUMENTS: PciVendorId = 0x104c;
    pub const PLX: PciVendorId = 0x10b5;
    pub const PERICOM: PciVendorId = 0x12d8;
    // Formerly ServerWorks
    pub const SERVERWORKS: PciVendorId = 0x1166;
    pub const O2_MICRO: PciVendorId = 0x1217;
    pub const RICOH: PciVendorId = 0x1180;
    pub const GENESYS_LOGIC: PciVendorId = 0x17a0;
    pub const WCH: PciVendorId = 0x1c00;

    /* FPGA Vendors */
    pub const XILINX: PciVendorId = 0x10ee;
    pub const ALTERA: PciVendorId = 0x1172;

    /// Returns the name of the vendor with the given ID. The kernel's own names take precedence so
    /// that otherwise identically named vendor IDs (e.g. the two Red Hat IDs) can be told apart,
    /// after which the PCI ID Repository is consulted.
    pub fn pci_vendor_id_to_str(vendor_id: PciVendorId) -> &'static str {
        match vendor_id {
            VENDOR_ID_UNKNOWN => "Unknown Vendor",
            VENDOR_ID_INVALID => "Invalid Vendor",
            INTEL => "Intel Corporation",
            AMD => "Advanced Micro Devices, Inc.",
            AMD_ATI => "Advanced Micro Devices, Inc. (ATI)",
            ARM => "ARM Ltd.",
            NVIDIA => "Nvidia Corporation",
            QEMU => "QEMU (Virtual Machine)",
            REDHAT_VIRTIO => "Red Hat, Inc. (VirtIO)",
            REDHAT => "Red Hat, Inc.",
            _ => super::super::pci_ids::vendor_name(vendor_id).unwrap_or("Unknown Vendor"),
        }
    }
}

pub mod device_id {
    use super::vendor_id::{self, PciVendorId};

    pub type PciDeviceId = u16;
    pub type PciVendorDeviceId = (PciVendorId, PciDeviceId);

    /* Devices missing from the PCI ID Repository */
    pub const QEMU_STD_VGA: PciVendorDeviceId = (vendor_id::QEMU, 0x1111);
    pub const QEMU_EDU: PciVendorDeviceId = (vendor_id::QEMU, 0x11e8);

    /* virtio 1.0+ (modern) devices, device ID = 0x1040 + virtio device type */
    pub const VIRTIO_NET: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1041);
    pub const VIRTIO_BLOCK: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1042);
    pub const VIRTIO_CONSOLE: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1043);
    pub const VIRTIO_RNG: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1044);
    pub const VIRTIO_BALLOON: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1045);
    pub const VIRTIO_SCSI: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1048);
    pub const VIRTIO_9P: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1049);
    pub const VIRTIO_GPU: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1050);
    pub const VIRTIO_INPUT: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1052);
    pub const VIRTIO_VSOCK: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1053);
    pub const VIRTIO_IOMMU: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1057);
    pub const VIRTIO_MEM: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x1058);
    pub const VIRTIO_FS: PciVendorDeviceId = (vendor_id::REDHAT_VIRTIO, 0x105a);

    /* QEMU emulated devices (Red Hat vendor ID) */
    pub const QEMU_PCI_BRIDGE: PciVendorDeviceId = (vendor_id::REDHAT, 0x0001);
    pub const QEMU_PCI_SERIAL: PciVendorDeviceId = (vendor_id::REDHAT, 0x0002);
    pub const QEMU_PCIE_HOST_BRIDGE: PciVendorDeviceId = (vendor_id::REDHAT, 0x0008);
    pub const QEMU_PCIE_ROOT_PORT: PciVendorDeviceId = (vendor_id::REDHAT, 0x000c);
    pub const QEMU_XHCI: PciVendorDeviceId = (vendor_id::REDHAT, 0x000d);
    pub const QEMU_PCIE_TO_PCI_BRIDGE: PciVendorDeviceId = (vendor_id::REDHAT, 0x000e);
    pub const QEMU_NVME: PciVendorDeviceId = (vendor_id::REDHAT, 0x0010);

    /// Returns the name of the device with the given vendor and device ID if it is known, first
    /// checking the devices defined above that are missing from the PCI ID Repository and then the
    /// repository itself.
    pub fn pci_device_id_to_str(
        vendor_id: PciVendorId,
        device_id: PciDeviceId,
    ) -> Option<&'static str> {
        match (vendor_id, device_id) {
            QEMU_STD_VGA => Some("QEMU/Bochs Standard VGA"),
            QEMU_EDU => Some("QEMU Educational Device"),
            _ => super::super::pci_ids::device_name(vendor_id, device_id),
        }
    }
}

pub mod device_class {
    pub type PciClassCode = u8;
    pub type PciSubclassCode = u8;
    pub type PciProgIf = u8;
    pub type PciClassFull = (PciClassCode, PciSubclassCode, PciProgIf);

    /* Unclassified Devices */
    pub const NON_VGA_UNCLASSIFIED: PciClassFull = (0x00, 0x00, 0x00);
    pub const VGA_COMPATIBLE_UNCLASSIFIED: PciClassFull = (0x00, 0x01, 0x00);
    pub const IMAGE_COPROCESSOR: PciClassFull = (0x00, 0x05, 0x00);

    /* Mass Storage Controllers */
    pub const SCSI_STORAGE_CONTROLLER: PciClassFull = (0x01, 0x00, 0x00);
    pub const SCSI_PQI_STORAGE_DEVICE: PciClassFull = (0x01, 0x00, 0x11);
    pub const SCSI_PQI_CONTROLLER: PciClassFull = (0x01, 0x00, 0x12);
    pub const SCSI_PQI_STORAGE_DEVICE_AND_CONTROLLER: PciClassFull = (0x01, 0x00, 0x13);
    pub const SCSI_NVME_STORAGE_DEVICE: PciClassFull = (0x01, 0x00, 0x21);
    pub const IDE_ISA_COMPAT_ONLY: PciClassFull = (0x01, 0x01, 0x00);
    pub const IDE_PCI_NATIVE_ONLY: PciClassFull = (0x01, 0x01, 0x05);
    pub const IDE_ISA_COMPAT_SWITCHABLE: PciClassFull = (0x01, 0x01, 0x0a);
    pub const IDE_PCI_NATIVE_SWITCHABLE: PciClassFull = (0x01, 0x01, 0x0f);
    pub const IDE_ISA_COMPAT_ONLY_BUS_MASTER: PciClassFull = (0x01, 0x01, 0x80);
    pub const IDE_PCI_NATIVE_ONLY_BUS_MASTER: PciClassFull = (0x01, 0x01, 0x85);
    pub const IDE_ISA_COMPAT_SWITCHABLE_BUS_MASTER: PciClassFull = (0x01, 0x01, 0x8a);
    pub const IDE_PCI_NATIVE_SWITCHABLE_BUS_MASTER: PciClassFull = (0x01, 0x01, 0x8f);
    pub const FLOPPY_DISK_CONTROLLER: PciClassFull = (0x01, 0x02, 0x00);
    pub const IPI_BUS_CONTROLLER: PciClassFull = (0x01, 0x03, 0x00);
    pub const RAID_BUS_CONTROLLER: PciClassFull = (0x01, 0x04, 0x00);
    pub const ATA_ADMA_SINGLE_STEPPING: PciClassFull = (0x01, 0x05, 0x20);
    pub const ATA_ADMA_CONTINUOUS: PciClassFull = (0x01, 0x05, 0x30);
    pub const SATA_VENDOR_SPECIFIC: PciClassFull = (0x01, 0x06, 0x00);
    pub const AHCI_SATA_CONTROLLER: PciClassFull = (0x01, 0x06, 0x01);
    pub const SATA_SERIAL_STORAGE_BUS: PciClassFull = (0x01, 0x06, 0x02);
    pub const SCSI_SAS_CONTROLLER: PciClassFull = (0x01, 0x07, 0x00);
    pub const SAS_SERIAL_STORAGE_BUS: PciClassFull = (0x01, 0x07, 0x01);
    pub const NVMHCI_CONTROLLER: PciClassFull = (0x01, 0x08, 0x01);
    pub const NVME_CONTROLLER: PciClassFull = (0x01, 0x08, 0x02);
    pub const NVME_ADMIN_CONTROLLER: PciClassFull = (0x01, 0x08, 0x03);
    pub const UFS_VENDOR_SPECIFIC: PciClassFull = (0x01, 0x09, 0x00);
    pub const UFSHCI_CONTROLLER: PciClassFull = (0x01, 0x09, 0x01);
    pub const OTHER_MASS_STORAGE_CONTROLLER: PciClassFull = (0x01, 0x80, 0x00);

    /* Network Controllers */
    pub const ETHERNET_CONTROLLER: PciClassFull = (0x02, 0x00, 0x00);
    pub const ETHERNET_IDPF_CONTROLLER: PciClassFull = (0x02, 0x00, 0x01);
    pub const TOKEN_RING_CONTROLLER: PciClassFull = (0x02, 0x01, 0x00);
    pub const FDDI_CONTROLLER: PciClassFull = (0x02, 0x02, 0x00);
    pub const ATM_CONTROLLER: PciClassFull = (0x02, 0x03, 0x00);
    pub const ISDN_CONTROLLER: PciClassFull = (0x02, 0x04, 0x00);
    pub const WORLDFIP_CONTROLLER: PciClassFull = (0x02, 0x05, 0x00);
    pub const PICMG_CONTROLLER: PciClassFull = (0x02, 0x06, 0x00);
    pub const INFINIBAND_NETWORK_CONTROLLER: PciClassFull = (0x02, 0x07, 0x00);
    pub const FABRIC_CONTROLLER: PciClassFull = (0x02, 0x08, 0x00);
    pub const OTHER_NETWORK_CONTROLLER: PciClassFull = (0x02, 0x80, 0x00);

    /* Display Controllers */
    pub const VGA_COMPATIBLE: PciClassFull = (0x03, 0x00, 0x00);
    pub const VGA_8514_COMPATIBLE: PciClassFull = (0x03, 0x00, 0x01);
    pub const XGA_COMPATIBLE: PciClassFull = (0x03, 0x01, 0x00);
    pub const DISPLAY_3D_CONTROLLER: PciClassFull = (0x03, 0x02, 0x00);
    pub const OTHER_DISPLAY_CONTROLLER: PciClassFull = (0x03, 0x80, 0x00);

    /* Multimedia Controllers */
    pub const MULTIMEDIA_VIDEO_CONTROLLER: PciClassFull = (0x04, 0x00, 0x00);
    pub const MULTIMEDIA_AUDIO_CONTROLLER: PciClassFull = (0x04, 0x01, 0x00);
    pub const COMPUTER_TELEPHONY_DEVICE: PciClassFull = (0x04, 0x02, 0x00);
    pub const HDA_AUDIO_CONTROLLER: PciClassFull = (0x04, 0x03, 0x00);
    pub const HDA_AUDIO_CONTROLLER_VENDOR_EXT: PciClassFull = (0x04, 0x03, 0x80);
    pub const OTHER_MULTIMEDIA_CONTROLLER: PciClassFull = (0x04, 0x80, 0x00);

    /* Memory Controllers */
    pub const RAM_CONTROLLER: PciClassFull = (0x05, 0x00, 0x00);
    pub const FLASH_CONTROLLER: PciClassFull = (0x05, 0x01, 0x00);
    pub const CXL_MEMORY_DEVICE_VENDOR_SPECIFIC: PciClassFull = (0x05, 0x02, 0x00);
    pub const CXL_MEMORY_DEVICE: PciClassFull = (0x05, 0x02, 0x10);
    pub const OTHER_MEMORY_CONTROLLER: PciClassFull = (0x05, 0x80, 0x00);

    /* Bridges */
    pub const HOST_BRIDGE: PciClassFull = (0x06, 0x00, 0x00);
    pub const PCI_TO_ISA_BRIDGE: PciClassFull = (0x06, 0x01, 0x00);
    pub const PCI_TO_EISA_BRIDGE: PciClassFull = (0x06, 0x02, 0x00);
    pub const PCI_TO_MICRO_CHANNEL_BRIDGE: PciClassFull = (0x06, 0x03, 0x00);
    pub const PCI_TO_PCI_BRIDGE: PciClassFull = (0x06, 0x04, 0x00);
    pub const PCI_TO_PCI_BRIDGE_SUB_DEC: PciClassFull = (0x06, 0x04, 0x01);
    pub const PCMCIA_BRIDGE: PciClassFull = (0x06, 0x05, 0x00);
    pub const NUBUS_BRIDGE: PciClassFull = (0x06, 0x06, 0x00);
    pub const CARDBUS_BRIDGE: PciClassFull = (0x06, 0x07, 0x00);
    pub const RACEWAY_BRIDGE_TRANSPARENT: PciClassFull = (0x06, 0x08, 0x00);
    pub const RACEWAY_BRIDGE_ENDPOINT: PciClassFull = (0x06, 0x08, 0x01);
    pub const SEMI_TRANSPARENT_PCI_BRIDGE_PRIMARY: PciClassFull = (0x06, 0x09, 0x40);
    pub const SEMI_TRANSPARENT_PCI_BRIDGE_SECONDARY: PciClassFull = (0x06, 0x09, 0x80);
    pub const INFINIBAND_TO_PCI_HOST_BRIDGE: PciClassFull = (0x06, 0x0a, 0x00);
    pub const OTHER_BRIDGE: PciClassFull = (0x06, 0x80, 0x00);

    /* Serial Controllers */
    pub const I8250: PciClassFull = (0x07, 0x00, 0x00);
    pub const NS16450: PciClassFull = (0x07, 0x00, 0x01);
    pub const NS16550: PciClassFull = (0x07, 0x00, 0x02);
    pub const NS16650: PciClassFull = (0x07, 0x00, 0x03);
    pub const NS16750: PciClassFull = (0x07, 0x00, 0x04);
    pub const NS16850: PciClassFull = (0x07, 0x00, 0x05);
    pub const NS16950: PciClassFull = (0x07, 0x00, 0x06);
    pub const NS16550_MULTI_PORT: PciClassFull = (0x07, 0x02, 0x02);
    pub const NS16650_MULTI_PORT: PciClassFull = (0x07, 0x02, 0x03);
    pub const NS16750_MULTI_PORT: PciClassFull = (0x07, 0x02, 0x04);
    pub const NS16850_MULTI_PORT: PciClassFull = (0x07, 0x02, 0x05);
    pub const NS16950_MULTI_PORT: PciClassFull = (0x07, 0x02, 0x06);

    /* Other Communication Controllers */
    pub const PARALLEL_SPP: PciClassFull = (0x07, 0x01, 0x00);
    pub const PARALLEL_BIDIRECTIONAL: PciClassFull = (0x07, 0x01, 0x01);
    pub const PARALLEL_ECP: PciClassFull = (0x07, 0x01, 0x02);
    pub const PARALLEL_IEEE1284_CONTROLLER: PciClassFull = (0x07, 0x01, 0x03);
    pub const PARALLEL_IEEE1284_TARGET: PciClassFull = (0x07, 0x01, 0xfe);
    pub const MODEM_GENERIC: PciClassFull = (0x07, 0x03, 0x00);
    pub const MODEM_HAYES_16450: PciClassFull = (0x07, 0x03, 0x01);
    pub const MODEM_HAYES_16550: PciClassFull = (0x07, 0x03, 0x02);
    pub const MODEM_HAYES_16650: PciClassFull = (0x07, 0x03, 0x03);
    pub const MODEM_HAYES_16750: PciClassFull = (0x07, 0x03, 0x04);
    pub const GPIB_CONTROLLER: PciClassFull = (0x07, 0x04, 0x00);
    pub const SMART_CARD_CONTROLLER: PciClassFull = (0x07, 0x05, 0x00);
    pub const OTHER_COMMUNICATION_CONTROLLER: PciClassFull = (0x07, 0x80, 0x00);

    /* Base System Peripherals */
    pub const PIC_8259: PciClassFull = (0x08, 0x00, 0x00);
    pub const PIC_ISA: PciClassFull = (0x08, 0x00, 0x01);
    pub const PIC_EISA: PciClassFull = (0x08, 0x00, 0x02);
    pub const IOAPIC: PciClassFull = (0x08, 0x00, 0x10);
    pub const IOXAPIC: PciClassFull = (0x08, 0x00, 0x20);
    pub const DMA_8237: PciClassFull = (0x08, 0x01, 0x00);
    pub const DMA_ISA: PciClassFull = (0x08, 0x01, 0x01);
    pub const DMA_EISA: PciClassFull = (0x08, 0x01, 0x02);
    pub const TIMER_8254: PciClassFull = (0x08, 0x02, 0x00);
    pub const TIMER_ISA: PciClassFull = (0x08, 0x02, 0x01);
    pub const TIMER_EISA: PciClassFull = (0x08, 0x02, 0x02);
    pub const HPET: PciClassFull = (0x08, 0x02, 0x03);
    pub const RTC_GENERIC: PciClassFull = (0x08, 0x03, 0x00);
    pub const RTC_ISA: PciClassFull = (0x08, 0x03, 0x01);
    pub const PCI_HOT_PLUG_CONTROLLER: PciClassFull = (0x08, 0x04, 0x00);
    pub const SD_HOST_CONTROLLER: PciClassFull = (0x08, 0x05, 0x00);
    // Primarily used for AMD's I/O Memory Management Unit (IOMMU) technology, but may be used by
    // other devices that provide similar functionality.
    pub const IOMMU: PciClassFull = (0x08, 0x06, 0x00);
    pub const ROOT_COMPLEX_EVENT_COLLECTOR: PciClassFull = (0x08, 0x07, 0x00);
    pub const OTHER_SYSTEM_PERIPHERAL: PciClassFull = (0x08, 0x80, 0x00);
    pub const TAP_TIMING_CARD: PciClassFull = (0x08, 0x99, 0x01);

    /* Input Device Controllers */
    pub const KEYBOARD_CONTROLLER: PciClassFull = (0x09, 0x00, 0x00);
    pub const DIGITIZER_PEN: PciClassFull = (0x09, 0x01, 0x00);
    pub const MOUSE_CONTROLLER: PciClassFull = (0x09, 0x02, 0x00);
    pub const SCANNER_CONTROLLER: PciClassFull = (0x09, 0x03, 0x00);
    pub const GAMEPORT_GENERIC: PciClassFull = (0x09, 0x04, 0x00);
    pub const GAMEPORT_EXTENDED: PciClassFull = (0x09, 0x04, 0x10);
    pub const OTHER_INPUT_CONTROLLER: PciClassFull = (0x09, 0x80, 0x00);

    /* Docking Stations */
    pub const GENERIC_DOCKING_STATION: PciClassFull = (0x0a, 0x00, 0x00);
    pub const OTHER_DOCKING_STATION: PciClassFull = (0x0a, 0x80, 0x00);

    /* Processors */
    pub const PROCESSOR_386: PciClassFull = (0x0b, 0x00, 0x00);
    pub const PROCESSOR_486: PciClassFull = (0x0b, 0x01, 0x00);
    pub const PROCESSOR_PENTIUM: PciClassFull = (0x0b, 0x02, 0x00);
    pub const PROCESSOR_ALPHA: PciClassFull = (0x0b, 0x10, 0x00);
    pub const PROCESSOR_POWERPC: PciClassFull = (0x0b, 0x20, 0x00);
    pub const PROCESSOR_MIPS: PciClassFull = (0x0b, 0x30, 0x00);
    pub const CO_PROCESSOR: PciClassFull = (0x0b, 0x40, 0x00);

    /* Serial Bus Controllers */
    pub const FIREWIRE_GENERIC: PciClassFull = (0x0c, 0x00, 0x00);
    pub const FIREWIRE_OHCI: PciClassFull = (0x0c, 0x00, 0x10);
    pub const ACCESS_BUS_CONTROLLER: PciClassFull = (0x0c, 0x01, 0x00);
    pub const SSA_CONTROLLER: PciClassFull = (0x0c, 0x02, 0x00);
    pub const USB_UHCI: PciClassFull = (0x0c, 0x03, 0x00);
    pub const USB_OHCI: PciClassFull = (0x0c, 0x03, 0x10);
    pub const USB_EHCI: PciClassFull = (0x0c, 0x03, 0x20);
    pub const USB_XHCI: PciClassFull = (0x0c, 0x03, 0x30);
    pub const USB4_ROUTER: PciClassFull = (0x0c, 0x03, 0x40);
    pub const USB_UNSPECIFIED: PciClassFull = (0x0c, 0x03, 0x80);
    pub const USB_DEVICE: PciClassFull = (0x0c, 0x03, 0xfe);
    pub const FIBRE_CHANNEL_CONTROLLER: PciClassFull = (0x0c, 0x04, 0x00);
    pub const SMBUS_CONTROLLER: PciClassFull = (0x0c, 0x05, 0x00);
    pub const INFINIBAND_CONTROLLER: PciClassFull = (0x0c, 0x06, 0x00);
    pub const IPMI_SMIC: PciClassFull = (0x0c, 0x07, 0x00);
    pub const IPMI_KCS: PciClassFull = (0x0c, 0x07, 0x01);
    pub const IPMI_BLOCK_TRANSFER: PciClassFull = (0x0c, 0x07, 0x02);
    pub const SERCOS_INTERFACE: PciClassFull = (0x0c, 0x08, 0x00);
    pub const CANBUS_CONTROLLER: PciClassFull = (0x0c, 0x09, 0x00);
    pub const MIPI_I3C_CONTROLLER: PciClassFull = (0x0c, 0x0a, 0x00);
    pub const OTHER_SERIAL_BUS_CONTROLLER: PciClassFull = (0x0c, 0x80, 0x00);

    /* Wireless Controllers */
    pub const IRDA_CONTROLLER: PciClassFull = (0x0d, 0x00, 0x00);
    pub const CONSUMER_IR_CONTROLLER: PciClassFull = (0x0d, 0x01, 0x00);
    pub const RF_CONTROLLER: PciClassFull = (0x0d, 0x10, 0x00);
    pub const BLUETOOTH_CONTROLLER: PciClassFull = (0x0d, 0x11, 0x00);
    pub const BROADBAND_CONTROLLER: PciClassFull = (0x0d, 0x12, 0x00);
    pub const WIFI_802_11A_CONTROLLER: PciClassFull = (0x0d, 0x20, 0x00);
    pub const WIFI_802_11B_CONTROLLER: PciClassFull = (0x0d, 0x21, 0x00);
    pub const CELLULAR_MODEM: PciClassFull = (0x0d, 0x40, 0x00);
    pub const CELLULAR_MODEM_WITH_ETHERNET: PciClassFull = (0x0d, 0x41, 0x00);
    pub const OTHER_WIRELESS_CONTROLLER: PciClassFull = (0x0d, 0x80, 0x00);

    /* Intelligent Controllers */
    pub const I2O_CONTROLLER: PciClassFull = (0x0e, 0x00, 0x00);

    /* Satellite Communications Controllers */
    pub const SATELLITE_TV_CONTROLLER: PciClassFull = (0x0f, 0x01, 0x00);
    pub const SATELLITE_AUDIO_CONTROLLER: PciClassFull = (0x0f, 0x02, 0x00);
    pub const SATELLITE_VOICE_CONTROLLER: PciClassFull = (0x0f, 0x03, 0x00);
    pub const SATELLITE_DATA_CONTROLLER: PciClassFull = (0x0f, 0x04, 0x00);

    /* Encryption Controllers */
    pub const NETWORK_AND_COMPUTING_ENCRYPTION: PciClassFull = (0x10, 0x00, 0x00);
    pub const ENTERTAINMENT_ENCRYPTION: PciClassFull = (0x10, 0x10, 0x00);
    pub const OTHER_ENCRYPTION_CONTROLLER: PciClassFull = (0x10, 0x80, 0x00);

    /* Signal Processing Controllers */
    pub const DPIO_MODULE: PciClassFull = (0x11, 0x00, 0x00);
    pub const PERFORMANCE_COUNTERS: PciClassFull = (0x11, 0x01, 0x00);
    pub const COMMUNICATION_SYNCHRONIZER: PciClassFull = (0x11, 0x10, 0x00);
    pub const SIGNAL_PROCESSING_MANAGEMENT: PciClassFull = (0x11, 0x20, 0x00);
    pub const OTHER_SIGNAL_PROCESSING_CONTROLLER: PciClassFull = (0x11, 0x80, 0x00);

    /* Processing Accelerators */
    pub const PROCESSING_ACCELERATOR: PciClassFull = (0x12, 0x00, 0x00);
    pub const SDXI_CONTROLLER: PciClassFull = (0x12, 0x01, 0x00);

    /* Miscellaneous */
    pub const NON_ESSENTIAL_INSTRUMENTATION: PciClassFull = (0x13, 0x00, 0x00);
    pub const COPROCESSOR: PciClassFull = (0x40, 0x00, 0x00);
    pub const UNASSIGNED_CLASS: PciClassFull = (0xff, 0x00, 0x00);
}
