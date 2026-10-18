use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::fmt::{self, Display, Formatter};

use hashbrown::HashMap;

use crate::cpu::isa::lp::ops::{
    config_read8,
    config_read16,
    config_read32,
    config_write8,
    config_write16,
    config_write32,
};
use crate::cpu::multiprocessor::spin::rwlock::{RwLock, RwLockWriteGuard};
use crate::device_management::drivers::busses::pci_express;
use crate::device_management::drivers::busses::pci_express::device_class::PciIdentifier;
use crate::device_management::drivers::busses::pci_express::device_class::vendor_id::VENDOR_ID_UNKNOWN;
use crate::device_management::drivers::busses::pci_express::ecam::capabilities::CapabilityIter;
use crate::device_management::drivers::busses::pci_express::ecam::headers::bridge::{
    CAPABILITIES_POINTER,
    PRIMARY_BUS_NUMBER,
    SECONDARY_BUS_NUMBER,
    SUBORDINATE_BUS_NUMBER,
};
use crate::device_management::drivers::busses::pci_express::ecam::headers::{
    CLASS,
    DEVICE_ID,
    HEADER_TYPE,
    PROG_IF,
    PcieCfgOffset,
    SUBCLASS,
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

/// Formats the location in the conventional `SSSS:BB:DD.F` notation.
impl Display for PcieLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04x}:{:02x}:{:02x}.{:x}",
            self.segment_group_num, self.bus_segment_num, self.device_num, self.function_num
        )
    }
}

/// Returns the keys of `map` in ascending order so that the topology is always logged in a
/// deterministic order regardless of the hash map's iteration order.
fn sorted_keys<K: Copy + Ord, V>(map: &HashMap<K, V>) -> Vec<K> {
    let mut keys: Vec<K> = map.keys().copied().collect();
    keys.sort_unstable();
    keys
}

#[derive(Debug)]
pub struct PcieTopology {
    segment_groups: HashMap<PcieSegmentGroupNum, RwLock<PcieSegmentGroup>>,
}

impl PcieTopology {
    fn segment_group_to_ecam_base(
        &self,
        segment_group_num: PcieSegmentGroupNum,
    ) -> Result<VirtualAddress, pci_express::Error> {
        self.segment_groups
            .get(&segment_group_num)
            .map(|sg| sg.read().ecam_base)
            .ok_or(pci_express::Error::InvalidSegmentGroup)
    }

    fn new(segment_group_info: Vec<PcieSegmentGroupInfo>) -> Self {
        let mut segment_groups = HashMap::new();
        for info in segment_group_info {
            segment_groups.insert(info.segment_group_num, RwLock::new(PcieSegmentGroup::new(info)));
        }
        Self {
            segment_groups,
        }
    }
}

/// Formats the whole PCIe topology as a tree with one line per segment group, bus segment and
/// function. Lines are separated by `\r\n` as required by `logln!` and the output has no trailing
/// line break, so it is intended to be logged with a single `logln!("{}", topology)`.
impl Display for PcieTopology {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "PCIe Topology ({} segment group(s)):", self.segment_groups.len())?;
        for sg_num in sorted_keys(&self.segment_groups) {
            write!(f, "{}", self.segment_groups[&sg_num].read())?;
        }
        Ok(())
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
    pub start_bus: PcieBusSegmentNum,
    pub end_bus: PcieBusSegmentNum,
    pub bus_segments: HashMap<PcieBusSegmentNum, PcieBusSegment>,
}

/// Each line written by this implementation is preceded by a line break so that it can be
/// directly appended to the line written by the enclosing `PcieTopology`.
impl Display for PcieSegmentGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\r\n  Segment Group {:04x} | buses {:02x}-{:02x} | ECAM @ {:?} | {} bus segment(s)",
            self.segment_group_num,
            self.start_bus,
            self.end_bus,
            self.ecam_base,
            self.bus_segments.len()
        )?;
        for bus_num in sorted_keys(&self.bus_segments) {
            write!(f, "{}", self.bus_segments[&bus_num])?;
        }
        Ok(())
    }
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
        ecam_base: VirtualAddress,
        start_bus: PcieBusSegmentNum,
        end_bus: PcieBusSegmentNum,
    ) -> HashMap<PcieBusSegmentNum, PcieBusSegment> {
        let mut bus_segments = HashMap::new();
        let mut bus_segs_to_enumerate = VecDeque::from([start_bus]);
        while let Some(bus_num) = bus_segs_to_enumerate.pop_front() {
            // Only enumerate bus segments within the current bus segment's decode range. Already
            // enumerated bus segments are skipped so that misconfigured bridges (e.g. ones whose
            // secondary bus number is still 0) can't cause the same bus to be enumerated forever.
            if bus_num >= start_bus && bus_num <= end_bus && !bus_segments.contains_key(&bus_num) {
                if let Some(mut bus_segment) =
                    PcieBusSegment::try_new(segment_group_num, ecam_base, bus_num)
                {
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
            start_bus: info.start_bus,
            end_bus: info.end_bus,
            bus_segments: Self::enumerate_bus_segments(
                info.segment_group_num,
                ecam_base,
                info.start_bus,
                info.end_bus,
            ),
        }
    }
}
#[derive(Debug)]
pub struct PcieBusSegment {
    pub bus_segment_num: PcieBusSegmentNum,
    pub devices: HashMap<PcieDeviceNum, RwLock<PcieDevice>>,
}

impl Display for PcieBusSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\r\n    Bus {:02x} | {} device(s)", self.bus_segment_num, self.devices.len())?;
        for device_num in sorted_keys(&self.devices) {
            write!(f, "{}", self.devices[&device_num].read())?;
        }
        Ok(())
    }
}

impl PcieBusSegment {
    pub fn try_new(
        segment_group_num: PcieSegmentGroupNum,
        ecam_base: VirtualAddress,
        bus_segment_num: PcieBusSegmentNum,
    ) -> Option<Self> {
        // Attempt to create a new PCIe bus segment by probing the ECAM for devices.
        // If no devices are found, return None.
        let mut devices = HashMap::new();
        for device_num in 0..MAX_PCIE_DEVICES_PER_BUS_SEGMENT as PcieDeviceNum {
            if let Some(device) =
                PcieDevice::try_new(segment_group_num, ecam_base, bus_segment_num, device_num)
            {
                devices.insert(device_num, RwLock::new(device));
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
            downstreams.extend(device.write().get_immediate_downstreams());
        }
        downstreams
    }
}

#[derive(Debug)]
pub struct PcieDevice {
    pub device_num: PcieDeviceNum,
    pub functions: Box<[RwLock<PcieFunction>]>,
}

/// Functions are written one per line since they are what actually carry the identifying
/// information; the device itself is only a grouping of functions sharing a device number.
impl Display for PcieDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for func in &self.functions {
            write!(f, "\r\n      {}", func.read())?;
        }
        Ok(())
    }
}

impl PcieDevice {
    pub fn try_new(
        segment_group_num: PcieSegmentGroupNum,
        ecam_base: VirtualAddress,
        bus_segment_num: PcieBusSegmentNum,
        device_num: PcieDeviceNum,
    ) -> Option<Self> {
        let mut functions = Vec::new();
        if let Some(func) = PcieFunction::try_get(
            ecam_base,
            PcieLocation {
                segment_group_num,
                bus_segment_num,
                device_num,
                function_num: 0,
            },
        ) {
            functions.push(RwLock::new(func));
            if functions[0].read().is_multifunction() {
                for function_num in 1..MAX_PCIE_FUNCTIONS_PER_DEVICE as PcieFunctionNum {
                    if let Some(func) = PcieFunction::try_get(
                        ecam_base,
                        PcieLocation {
                            segment_group_num,
                            bus_segment_num,
                            device_num,
                            function_num,
                        },
                    ) {
                        functions.push(RwLock::new(func));
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
            if let PcieFunctionType::Bridge(bridge) = PcieFunctionType::from(func.write()) {
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
    pub location: PcieLocation,
    cfg_base: VirtualAddress,
}

/// Writes a single line describing the function: its location, header type, the identifying
/// information along with the device interface this kernel recognizes it as, and for bridges the
/// bus segment numbers they connect.
impl Display for PcieFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_bridge() {
            write!(
                f,
                "{} [bridge]   {} primary bus {:02x}, secondary bus {:02x}, subordinate bus {:02x}",
                self.location,
                self.identifier(),
                self.cfg_read8(PRIMARY_BUS_NUMBER.offset),
                self.cfg_read8(SECONDARY_BUS_NUMBER.offset),
                self.cfg_read8(SUBORDINATE_BUS_NUMBER.offset)
            )
        } else {
            write!(f, "{} [endpoint] {}", self.location, self.identifier())
        }
    }
}

impl PcieFunction {
    pub fn cfg_read8(&self, offset: PcieCfgOffset) -> u8 {
        unsafe { config_read8(self.cfg_base + <PcieCfgOffset as Into<usize>>::into(offset)) }
    }

    pub fn cfg_read16(&self, offset: PcieCfgOffset) -> u16 {
        unsafe { config_read16(self.cfg_base + <PcieCfgOffset as Into<usize>>::into(offset)) }
    }

    pub fn cfg_read32(&self, offset: PcieCfgOffset) -> u32 {
        unsafe { config_read32(self.cfg_base + <PcieCfgOffset as Into<usize>>::into(offset)) }
    }

    pub fn cfg_write8(&mut self, offset: PcieCfgOffset, value: u8) {
        unsafe {
            config_write8(self.cfg_base + <PcieCfgOffset as Into<usize>>::into(offset), value)
        }
    }

    pub fn cfg_write16(&mut self, offset: PcieCfgOffset, value: u16) {
        unsafe {
            config_write16(self.cfg_base + <PcieCfgOffset as Into<usize>>::into(offset), value)
        }
    }

    pub fn cfg_write32(&mut self, offset: PcieCfgOffset, value: u32) {
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

    pub fn identifier(&self) -> PciIdentifier {
        PciIdentifier {
            vendor_id: self.cfg_read16(VENDOR_ID.offset),
            device_id: self.cfg_read16(DEVICE_ID.offset),
            class_code: self.cfg_read8(CLASS.offset),
            subclass: self.cfg_read8(SUBCLASS.offset),
            prog_if: self.cfg_read8(PROG_IF.offset),
        }
    }

    pub fn get_capability_offset(&self) -> PcieCfgOffset {
        self.cfg_read8(CAPABILITIES_POINTER.offset).into()
    }

    pub fn get_capability_iter<'a>(&'a mut self) -> CapabilityIter<'a> {
        CapabilityIter::new(self)
    }

    /// The ECAM base is passed in rather than looked up in `PCIE_TOPOLOGY` because functions are
    /// probed while `PCIE_TOPOLOGY` itself is still being initialized, and accessing it from
    /// within its own initializer would deadlock.
    fn try_get(ecam_base: VirtualAddress, loc: PcieLocation) -> Option<Self> {
        let cfg_base =
            compute_cfg_base(ecam_base, loc.bus_segment_num, loc.device_num, loc.function_num);
        let func = Self {
            location: loc,
            cfg_base,
        };
        if func.is_present() {
            Some(func)
        } else {
            None
        }
    }
}

pub struct PcieBridge<'a>(RwLockWriteGuard<'a, PcieFunction>);

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
pub struct PcieEndpoint<'a>(RwLockWriteGuard<'a, PcieFunction>);

impl<'a> PcieEndpoint<'a> {
    const CAP_EXT_BASE: PcieCfgOffset = PcieCfgOffset::new_const(256);
}

pub enum PcieFunctionType<'a> {
    Bridge(PcieBridge<'a>),
    Endpoint(PcieEndpoint<'a>),
}

impl<'a> From<RwLockWriteGuard<'a, PcieFunction>> for PcieFunctionType<'a> {
    fn from(func: RwLockWriteGuard<'a, PcieFunction>) -> Self {
        if func.is_bridge() {
            PcieFunctionType::Bridge(PcieBridge(func))
        } else {
            PcieFunctionType::Endpoint(PcieEndpoint(func))
        }
    }
}
