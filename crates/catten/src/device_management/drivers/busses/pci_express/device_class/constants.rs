pub mod vendor_id {
    pub type PciVendorId = u16;

    pub const VENDOR_ID_UNKNOWN: PciVendorId = 0xffff;
    pub const VENDOR_ID_INVALID: PciVendorId = 0x0000;

    pub const INTEL: PciVendorId = 0x8086;
    pub const AMD: PciVendorId = 0x1022;
    pub const ARM: PciVendorId = 0x13b5;
    pub const NVIDIA: PciVendorId = 0x10de;
    pub const QEMU: PciVendorId = 0x1234;
    // Red Hat, Inc. (primarily used for virtio devices)
    pub const REDHAT_VIRTIO: PciVendorId = 0x1af4;
    pub const REDHAT: PciVendorId = 0x1b36;

    pub const fn pci_vendor_id_to_str(vendor_id: PciVendorId) -> &'static str {
        match vendor_id {
            VENDOR_ID_UNKNOWN => "Unknown Vendor",
            VENDOR_ID_INVALID => "Invalid Vendor",
            INTEL => "Intel Corporation",
            AMD => "Advanced Micro Devices, Inc.",
            ARM => "ARM Ltd.",
            NVIDIA => "Nvidia Corporation",
            QEMU => "QEMU (Virtual Machine)",
            REDHAT_VIRTIO => "Red Hat, Inc. (VirtIO)",
            REDHAT => "Red Hat, Inc.",
            _ => "Unknown Vendor",
        }
    }
}

pub mod device_class {
    pub type PciClassCode = u8;
    pub type PciSubclassCode = u8;
    pub type PciProgIf = u8;
    pub type PciClassFull = (PciClassCode, PciSubclassCode, PciProgIf);

    /* Mass Storage Controllers */
    pub const AHCI_SATA_CONTROLLER: PciClassFull = (0x01, 0x06, 0x01);
    pub const SCSI_SAS_CONTROLLER: PciClassFull = (0x01, 0x07, 0x00);
    pub const NVME_CONTROLLER: PciClassFull = (0x01, 0x08, 0x02);

    /* Display Controllers */
    pub const VGA_COMPATIBLE: PciClassFull = (0x03, 0x00, 0x00);
    pub const OTHER_DISPLAY_CONTROLLER: PciClassFull = (0x03, 0x80, 0x00);

    /* Bridges */
    pub const HOST_BRIDGE: PciClassFull = (0x06, 0x00, 0x00);
    pub const PCI_TO_ISA_BRIDGE: PciClassFull = (0x06, 0x01, 0x00);
    pub const PCI_TO_PCI_BRIDGE: PciClassFull = (0x06, 0x04, 0x00);
    pub const PCI_TO_PCI_BRIDGE_SUB_DEC: PciClassFull = (0x06, 0x04, 0x01);

    /* NS16x50 UARTs */
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

    /* Base System Peripherals */
    pub const IOAPIC: PciClassFull = (0x08, 0x00, 0x10);
    pub const IOXAPIC: PciClassFull = (0x08, 0x00, 0x20);
    pub const HPET: PciClassFull = (0x08, 0x02, 0x03);
    pub const SD_HOST_CONTROLLER: PciClassFull = (0x08, 0x05, 0x00);
    // Primarily used for AMD's I/O Memory Management Unit (IOMMU) technology, but may be used by
    // other devices that provide similar functionality.
    pub const IOMMU: PciClassFull = (0x08, 0x06, 0x00);

    /* Serial Bus Controllers */
    pub const USB_EHCI: PciClassFull = (0x0c, 0x03, 0x20);
    pub const USB_XHCI: PciClassFull = (0x0c, 0x03, 0x30);
    pub const USB4_ROUTER: PciClassFull = (0x0c, 0x03, 0x40);
    pub const SMBUS_CONTROLLER: PciClassFull = (0x0c, 0x05, 0x00);
    pub const IPMI_KCS: PciClassFull = (0x0c, 0x06, 0x01);
}
