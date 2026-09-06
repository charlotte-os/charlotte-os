use core::mem::ManuallyDrop;

use crate::device_management::drivers::busses::pci_express::device_class::PciIdentifier;
use crate::device_management::drivers::busses::pci_express::ecam::capabilities::standard::PciCapabilityOffset;

#[repr(C, packed)]
/// The Common portion of the PCIe configuration space header; shared by both endpoint and bridge
/// devices
pub struct CfgCommonHeader {
    vendor_id: u16,
    device_id: u16,
    command: u16,
    status: u16,
    revision_id: u8,
    prog_if: u8,
    subclass: u8,
    class_code: u8,
    cache_line_size: u8,
    latency_timer: u8,
    header_type: u8,
    bist: u8,
}

impl CfgCommonHeader {
    const HEADER_TYPE_MASK: u8 = 0b1;
    /* Source: https://wiki.osdev.org/PCI#Configuration_Space */
    const HEADER_TYPE_SINGLE_FUNC_MASK: u8 = 0b1 << 7;
    const VENDOR_ID_NOT_PRESENT: u16 = 0xffff;

    pub fn is_device_present(&self) -> bool {
        self.vendor_id != Self::VENDOR_ID_NOT_PRESENT
    }

    pub fn is_bridge(&self) -> bool {
        self.header_type & Self::HEADER_TYPE_MASK == 0b1
    }

    pub fn is_multi_function(&self) -> bool {
        self.header_type & Self::HEADER_TYPE_SINGLE_FUNC_MASK != 0
    }

    pub fn get_identifier(&self) -> PciIdentifier {
        PciIdentifier {
            vendor_id: self.vendor_id,
            device_id: self.device_id,
            class_code: self.class_code,
            subclass: self.subclass,
            prog_if: self.prog_if,
        }
    }

    /// Determines if the PCI(-X/e) device supports capabilities.
    pub fn are_capabilities_supported(&self) -> bool {
        const CAPABILITIES_SUPPORT_STATUS_BIT: u16 = 1 << 4;

        self.status & CAPABILITIES_SUPPORT_STATUS_BIT != 0
    }
}

#[repr(C, packed)]
/// The configuration space header for PCIe bridge devices, which extends the common header with
/// bridge-specific fields
pub struct CfgBridgeHeader {
    common: CfgCommonHeader,
    bars: [u32; 2],
    primary_bus_num: u8,
    secondary_bus_num: u8,
    subordinate_bus_num: u8,
    secondary_latency_timer: u8,
    io_base: u8,
    io_limit: u8,
    secondary_status: u16,
    memory_base: u16,
    memory_limit: u16,
    prefetchable_memory_base: u16,
    prefetchable_memory_limit: u16,
    prefetchable_base_upper32: u32,
    prefetchable_limit_upper32: u32,
    io_base_upper16: u16,
    io_limit_upper16: u16,
    capabilities_offset: u8,
    unused0: [u8; 7],
    interrupt_line: u8,
    interrupt_pin: u8,
    bridge_control: u16,
}

impl CfgBridgeHeader {
    pub fn get_secondary_bus_num(&self) -> u8 {
        self.secondary_bus_num
    }

    pub fn is_secondary_bus_pcie(&self) -> bool {
        todo!(
            "Implement this by checking for PCIe capability in the capability list of the \
             bridge's configuration space."
        )
    }

    pub fn get_capabilities_offset(&self) -> Option<PciCapabilityOffset> {
        if self.common.are_capabilities_supported() {
            Some(self.capabilities_offset)
        } else {
            None
        }
    }
}

#[repr(C, packed)]
/// The configuration space header for PCIe endpoint devices, which extends the common header with
/// endpoint-specific fields
pub struct CfgEndpointHeader {
    common: CfgCommonHeader,
    bars: [u32; 6],
    _cardbus_cis_ptr: u32,
    subsystem_vendor_id: u16,
    subsystem_id: u16,
    expansion_rom_base_addr: u32,
    capabilities_offset: u8,
    _unused0: [u8; 7],
    interrupt_line: u8,
    interrupt_pin: u8,
    _min_grant: u8,
    _max_latency: u8,
}

impl CfgEndpointHeader {
    pub fn get_capabilities_offset(&self) -> Option<PciCapabilityOffset> {
        if self.common.are_capabilities_supported() {
            Some(self.capabilities_offset)
        } else {
            None
        }
    }
}

/// The configuration space header for a PCIe device, which can be either a bridge or an endpoint
pub union CfgHeader {
    pub common: ManuallyDrop<CfgCommonHeader>, /* For determining header type before safely
                                                * accessing bridge/endpoint-specific fields */
    pub bridge: ManuallyDrop<CfgBridgeHeader>,
    pub endpoint: ManuallyDrop<CfgEndpointHeader>,
}
