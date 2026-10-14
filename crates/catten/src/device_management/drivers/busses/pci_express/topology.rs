use alloc::boxed::Box;
use alloc::collections::VecDeque;

use hashbrown::HashMap;

use crate::device_management::drivers::busses::pci_express::ecam::headers::PcieCfgOffset;
use crate::klib::size::mebibytes;
use crate::logln;
use crate::memory::allocators::memory::PageSize;
use crate::memory::linear::MemoryMapping;
use crate::memory::linear::address_map::{LA_MAP, LinearMemoryRegion, RegionType};
use crate::memory::{AddressSpaceInterface, KERNEL_AS, PhysicalAddress, VirtualAddress};

pub type PcieSegmentGroupNum = u16;
pub type PcieBusSegmentNum = u8;
pub type PcieDeviceNum = u8;
pub type PcieFunctionNum = u8;

const MAX_PCIE_SEGMENT_GROUPS: usize = 256;
const MAX_PCIE_BUS_SEGMENTS_PER_SEGMENT_GROUP: usize = 256;
const MAX_PCIE_DEVICES_PER_BUS_SEGMENT: usize = 32;
const MAX_PCIE_FUNCTIONS_PER_DEVICE: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PcieLocation {
    pub segment_group_num: PcieSegmentGroupNum,
    pub bus_segment_num: PcieBusSegmentNum,
    pub device_num: PcieDeviceNum,
    pub function_num: PcieFunctionNum,
}

pub struct PcieTopology {
    segment_groups: HashMap<PcieSegmentGroupNum, PcieSegmentGroup>,
}

pub struct PcieSegmentGroup {
    pub segment_group_num: PcieSegmentGroupNum,
    ecam_base: VirtualAddress,
    pub bus_segments: HashMap<PcieBusSegmentNum, PcieBusSegment>,
}

impl PcieSegmentGroup {
    const ECAM_SIZE: usize = mebibytes(256);

    /* Each PCIe segment group's ECAM occupies 256 MiB of address space */

    pub(super) fn map_ecam(base: PhysicalAddress) -> VirtualAddress {
        logln!("[drivers::busses::pci_express] Mapping PCIe ECAM at physical address {:?}", base);
        let mut kas = KERNEL_AS.lock();
        logln!(
            "[drivers::busses::pci_express] Finding free virtual address range for PCIe ECAM \
             mapping of size {:?} bytes",
            (Self::ECAM_SIZE)
        );
        let vbase = kas
            .find_free_region_large_aligned(
                Self::ECAM_SIZE / PageSize::Large.num_bytes(),
                <LinearMemoryRegion as Into<(VirtualAddress, VirtualAddress)>>::into(
                    *LA_MAP.get_region(RegionType::KernelMmio),
                ),
            )
            .expect("Failed to find free virtual address range for PCIe ECAM mapping");
        logln!(
            "[drivers::busses::pci_express] Mapping PCIe ECAM at physical address {:?} to virtual \
             address {:?}",
            base,
            vbase
        );
        let mut mem_mapping: MemoryMapping;
        for offset in (0..Self::ECAM_SIZE).step_by(mebibytes(2)) {
            mem_mapping = MemoryMapping {
                vaddr: vbase + offset,
                paddr: base + offset,
                page_type: crate::memory::linear::PageType::Mmio,
            };
            kas.map_large_page(mem_mapping).expect("Failed to map large page for a PCIe ECAM!");
        }
        logln!(
            "[drivers::busses::pci_express] Successfully mapped PCIe ECAM at physical address \
             {:?} to virtual address {:?}",
            base,
            vbase
        );
        vbase
    }

    fn enumerate_bus_segments(
        ecam_base: VirtualAddress,
        start_bus: PcieBusSegmentNum,
        end_bus: PcieBusSegmentNum,
    ) -> HashMap<PcieBusSegmentNum, PcieBusSegment> {
        let mut bus_segments = HashMap::new();
        let mut bus_segs_to_enumerate = VecDeque::from([start_bus]);
        for bus_num in bus_segs_to_enumerate {
            if let Some(bus_segment) = PcieBusSegment::try_new(ecam_base, bus_num) {
                bus_segments.insert(bus_num, bus_segment);
            }
        }
        bus_segments
    }

    fn new(
        segment_group_num: PcieSegmentGroupNum,
        phys_base: PhysicalAddress,
        start_bus: PcieBusSegmentNum,
        end_bus: PcieBusSegmentNum,
    ) -> Self {
        let ecam_base = Self::map_ecam(phys_base);
        Self {
            segment_group_num,
            ecam_base,
            bus_segments: Self::enumerate_bus_segments(ecam_base, start_bus, end_bus),
        }
    }
}

pub struct PcieBusSegment {
    pub bus_segment_num: PcieBusSegmentNum,
    pub devices: HashMap<PcieDeviceNum, PcieDevice>,
}

impl PcieBusSegment {
    pub fn try_new(ecam_base: VirtualAddress, bus_segment_num: PcieBusSegmentNum) -> Option<Self> {
        // Attempt to create a new PCIe bus segment by probing the ECAM for devices.
        // If no devices are found, return None.
        let mut devices = HashMap::new();
        for device_num in 0..=31 {
            if let Some(device) = PcieDevice::try_new(ecam_base, bus_segment_num, device_num) {
                devices.insert(device_num, device);
            }
        }
        if devices.is_empty() {
            None
        } else {
            Some(Self {
                bus_segment_num,
                devices,
            })
        }
    }
}

pub struct PcieDevice {
    pub device_num: PcieDeviceNum,
    pub functions: Box<[PcieFunction]>,
}

pub enum PcieFunction {
    Bridge(PcieBridge),
    Endpoint(PcieEndpoint),
}

impl PcieFunction {
    pub fn is_present(&self) -> bool {}
}

pub struct PcieBridge {
    cfg_base: VirtualAddress,
    // Parent bus segment for this bridge.
    primary_bus_segment_num: PcieBusSegmentNum,
    // The starting bus segment number for the secondary bus behind this bridge.
    secondary_bus_segment_num: PcieBusSegmentNum,
    // The highest bus segment number that is subordinate to this bridge.
    subordinate_bus_segment_num: PcieBusSegmentNum,
}
pub struct PcieEndpoint {
    cfg_base: VirtualAddress,
    cap_base: PcieCfgOffset,
}
