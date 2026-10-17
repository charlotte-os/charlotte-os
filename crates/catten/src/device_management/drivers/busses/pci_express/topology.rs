use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use hashbrown::HashMap;

use crate::cpu::isa::lp::ops::{
    config_read8,
    config_read16,
    config_read32,
    config_write8,
    config_write16,
    config_write32,
};
use crate::cpu::multiprocessor::spin::rwlock::RwLock;
use crate::device_management::drivers::busses::pci_express;
use crate::device_management::drivers::busses::pci_express::device_class::vendor_id::VENDOR_ID_UNKNOWN;
use crate::device_management::drivers::busses::pci_express::ecam::headers::bridge::{
    PRIMARY_BUS_NUMBER,
    SECONDARY_BUS_NUMBER,
    SUBORDINATE_BUS_NUMBER,
};
use crate::device_management::drivers::busses::pci_express::ecam::headers::{
    HEADER_TYPE,
    PcieCfgOffset,
    VENDOR_ID,
};
use crate::klib::size::mebibytes;
use crate::memory::allocators::memory::PageSize;
use crate::memory::linear::MemoryMapping;
use crate::memory::linear::address_map::{LA_MAP, LinearMemoryRegion, RegionType};
use crate::memory::{AddressSpaceInterface, KERNEL_AS, PhysicalAddress, VirtualAddress};
use crate::{environment, logln};

pub type PcieSegmentGroupNum = u16;
pub type PcieBusSegmentNum = u8;
pub type PcieDeviceNum = u8;
pub type PcieFunctionNum = u8;

const MAX_PCIE_SEGMENT_GROUPS: usize = 256;
const MAX_PCIE_BUS_SEGMENTS_PER_SEGMENT_GROUP: usize = 256;
const MAX_PCIE_DEVICES_PER_BUS_SEGMENT: usize = 32;
const MAX_PCIE_FUNCTIONS_PER_DEVICE: usize = 8;

pub static PCIE_TOPOLOGY: spin::LazyLock<PcieTopology> =
    spin::LazyLock::new(|| PcieTopology::new(environment::get_pcie_segment_groups()));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PcieLocation {
    pub segment_group_num: PcieSegmentGroupNum,
    pub bus_segment_num: PcieBusSegmentNum,
    pub device_num: PcieDeviceNum,
    pub function_num: PcieFunctionNum,
}

#[derive(Debug)]
pub struct PcieTopology {
    segment_groups: HashMap<PcieSegmentGroupNum, PcieSegmentGroup>,
}

impl PcieTopology {
    fn segment_group_to_ecam_base(
        &self,
        segment_group_num: PcieSegmentGroupNum,
    ) -> Result<VirtualAddress, pci_express::Error> {
        self.segment_groups
            .get(&segment_group_num)
            .map(|sg| sg.ecam_base)
            .ok_or(pci_express::Error::InvalidSegmentGroup)
    }

    fn new(segment_group_info: Vec<PcieSegmentGroupInfo>) -> Self {
        let mut segment_groups = HashMap::new();
        for info in segment_group_info {
            segment_groups.insert(info.segment_group_num, PcieSegmentGroup::new(info));
        }
        Self {
            segment_groups,
        }
    }
}

pub struct PcieSegmentGroupInfo {
    pub segment_group_num: PcieSegmentGroupNum,
    pub ecam_base: PhysicalAddress,
    pub start_bus: PcieBusSegmentNum,
    pub end_bus: PcieBusSegmentNum,
}

#[derive(Debug)]
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
        segment_group_num: PcieSegmentGroupNum,
        start_bus: PcieBusSegmentNum,
        end_bus: PcieBusSegmentNum,
    ) -> HashMap<PcieBusSegmentNum, PcieBusSegment> {
        let mut bus_segments = HashMap::new();
        let mut bus_segs_to_enumerate = VecDeque::from([start_bus]);
        while let Some(bus_num) = bus_segs_to_enumerate.pop_front() {
            // Only enumerate bus segments within the current bus segment's decode range
            if bus_num >= start_bus && bus_num <= end_bus {
                if let Some(mut bus_segment) = PcieBusSegment::try_new(segment_group_num, bus_num) {
                    bus_segs_to_enumerate.extend((&mut bus_segment).get_immediate_downstreams());
                    bus_segments.insert(bus_num, bus_segment);
                }
            }
        }
        bus_segments
    }

    fn new(info: PcieSegmentGroupInfo) -> Self {
        let ecam_base = Self::map_ecam(info.ecam_base);
        Self {
            segment_group_num: info.segment_group_num,
            ecam_base,
            bus_segments: Self::enumerate_bus_segments(
                info.segment_group_num,
                info.start_bus,
                info.end_bus,
            ),
        }
    }
}
#[derive(Debug)]
pub struct PcieBusSegment {
    pub bus_segment_num: PcieBusSegmentNum,
    pub devices: HashMap<PcieDeviceNum, PcieDevice>,
}

impl PcieBusSegment {
    pub fn try_new(
        segment_group_num: PcieSegmentGroupNum,
        bus_segment_num: PcieBusSegmentNum,
    ) -> Option<Self> {
        // Attempt to create a new PCIe bus segment by probing the ECAM for devices.
        // If no devices are found, return None.
        let mut devices = HashMap::new();
        for device_num in 0..=31 {
            if let Some(device) =
                PcieDevice::try_new(segment_group_num, bus_segment_num, device_num)
            {
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

    fn get_immediate_downstreams(&mut self) -> Vec<PcieBusSegmentNum> {
        let mut downstreams = Vec::new();
        for device in self.devices.values_mut() {
            downstreams.extend(device.get_immediate_downstreams());
        }
        downstreams
    }
}

#[derive(Debug)]
pub struct PcieDevice {
    pub device_num: PcieDeviceNum,
    pub functions: Box<[PcieFunction]>,
}

impl PcieDevice {
    pub fn try_new(
        segment_group_num: PcieSegmentGroupNum,
        bus_segment_num: PcieBusSegmentNum,
        device_num: PcieDeviceNum,
    ) -> Option<Self> {
        let mut functions = Vec::new();
        if let Some(func) = PcieFunction::try_get(PcieLocation {
            segment_group_num,
            bus_segment_num,
            device_num,
            function_num: 0,
        }) {
            functions.push(func);
            if functions[0].is_multifunction() {
                for function_num in 1..=7 {
                    if let Some(func) = PcieFunction::try_get(PcieLocation {
                        segment_group_num,
                        bus_segment_num,
                        device_num,
                        function_num,
                    }) {
                        functions.push(func);
                    }
                }
            }
        }
        if functions.is_empty() {
            None
        } else {
            Some(Self {
                device_num,
                functions: functions.into_boxed_slice(),
            })
        }
    }

    fn get_immediate_downstreams(&mut self) -> Vec<PcieBusSegmentNum> {
        let mut downstreams = Vec::new();
        for func in &mut self.functions {
            if let PcieFunctionType::Bridge(bridge) = PcieFunctionType::from(func) {
                downstreams.push(bridge.downstream_start_bus_seg());
            }
        }
        downstreams
    }
}

fn compute_cfg_base(
    ecam_base: VirtualAddress,
    bus_segment_num: PcieBusSegmentNum,
    device_num: PcieDeviceNum,
    function_num: u8,
) -> VirtualAddress {
    ecam_base
        + ((bus_segment_num as usize) << 20)
        + ((device_num as usize) << 15)
        + ((function_num as usize) << 12)
}

#[derive(Debug)]
pub struct PcieFunction {
    cfg_base: VirtualAddress,
}

impl PcieFunction {
    fn cfg_read8(&self, offset: PcieCfgOffset) -> u8 {
        unsafe { config_read8(self.cfg_base + <PcieCfgOffset as Into<usize>>::into(offset)) }
    }

    fn cfg_read16(&self, offset: PcieCfgOffset) -> u16 {
        unsafe { config_read16(self.cfg_base + <PcieCfgOffset as Into<usize>>::into(offset)) }
    }

    fn cfg_read32(&self, offset: PcieCfgOffset) -> u32 {
        unsafe { config_read32(self.cfg_base + <PcieCfgOffset as Into<usize>>::into(offset)) }
    }

    fn cfg_write8(&mut self, offset: PcieCfgOffset, value: u8) {
        unsafe {
            config_write8(self.cfg_base + <PcieCfgOffset as Into<usize>>::into(offset), value)
        }
    }

    fn cfg_write16(&mut self, offset: PcieCfgOffset, value: u16) {
        unsafe {
            config_write16(self.cfg_base + <PcieCfgOffset as Into<usize>>::into(offset), value)
        }
    }

    fn cfg_write32(&mut self, offset: PcieCfgOffset, value: u32) {
        unsafe {
            config_write32(self.cfg_base + <PcieCfgOffset as Into<usize>>::into(offset), value)
        }
    }

    pub fn is_present(&self) -> bool {
        self.cfg_read16(VENDOR_ID.offset) != VENDOR_ID_UNKNOWN
    }

    pub fn is_bridge(&self) -> bool {
        self.cfg_read8(HEADER_TYPE.offset) & 0b0111_1111 == 1
    }

    pub fn is_multifunction(&self) -> bool {
        self.cfg_read8(HEADER_TYPE.offset) & (1 << 7) != 0
    }

    fn try_get(loc: PcieLocation) -> Option<Self> {
        let cfg_base = compute_cfg_base(
            PCIE_TOPOLOGY.segment_group_to_ecam_base(loc.segment_group_num).ok()?,
            loc.bus_segment_num,
            loc.device_num,
            loc.function_num,
        );
        let func = Self {
            cfg_base,
        };
        if func.is_present() {
            Some(func)
        } else {
            None
        }
    }
}

pub struct PcieBridge<'a>(&'a mut PcieFunction);

impl<'a> PcieBridge<'a> {
    fn upstream_bus_seg(&self) -> PcieBusSegmentNum {
        self.0.cfg_read8(PRIMARY_BUS_NUMBER.offset)
    }

    fn downstream_start_bus_seg(&self) -> PcieBusSegmentNum {
        self.0.cfg_read8(SECONDARY_BUS_NUMBER.offset)
    }

    fn downstream_end_bus_seg(&self) -> PcieBusSegmentNum {
        self.0.cfg_read8(SUBORDINATE_BUS_NUMBER.offset)
    }
}
pub struct PcieEndpoint<'a>(&'a mut PcieFunction);

impl<'a> PcieEndpoint<'a> {
    const CAP_EXT_BASE: PcieCfgOffset = PcieCfgOffset::new_const(256);
}

pub enum PcieFunctionType<'a> {
    Bridge(PcieBridge<'a>),
    Endpoint(PcieEndpoint<'a>),
}

impl<'a> From<&'a mut PcieFunction> for PcieFunctionType<'a> {
    fn from(func: &'a mut PcieFunction) -> Self {
        if func.is_bridge() {
            PcieFunctionType::Bridge(PcieBridge(func))
        } else {
            PcieFunctionType::Endpoint(PcieEndpoint(func))
        }
    }
}
