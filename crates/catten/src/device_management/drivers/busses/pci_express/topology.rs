use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ops::Deref;
use core::ptr::NonNull;

use crate::cpu::isa::interface::memory::address::VirtualAddressIfce;
use crate::cpu::multiprocessor::spin::mutex::Mutex as SpinMutex;
use crate::device_management::drivers::busses::pci_express::device_class::PciIdentifier;
use crate::device_management::drivers::busses::pci_express::ecam::pcie::PcieCfgSpace;
use crate::device_management::drivers::busses::pci_express::{
    Error,
    MAX_DEVICES_PER_BUS,
    MAX_FUNCTIONS_PER_DEVICE,
    ecam,
};
use crate::logln;
use crate::memory::{PhysicalAddress, VirtualAddress};

pub type PcieSegmentGroupNum = u16;
pub type PcieBusSegmentNum = u8;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PcieDeviceNum(u8);
impl TryFrom<u8> for PcieDeviceNum {
    type Error = ();

    fn try_from(num: u8) -> Result<Self, Self::Error> {
        if num < MAX_DEVICES_PER_BUS as u8 {
            Ok(PcieDeviceNum(num))
        } else {
            Err(())
        }
    }
}
impl PcieDeviceNum {
    pub fn get_inner(self) -> u8 {
        self.0
    }
}
impl Deref for PcieDeviceNum {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PcieFunctionNum(u8);
impl TryFrom<u8> for PcieFunctionNum {
    type Error = ();

    fn try_from(num: u8) -> Result<Self, Self::Error> {
        if num < MAX_FUNCTIONS_PER_DEVICE as u8 {
            Ok(PcieFunctionNum(num))
        } else {
            Err(())
        }
    }
}
impl PcieFunctionNum {
    pub fn get_inner(self) -> u8 {
        self.0
    }
}
impl Deref for PcieFunctionNum {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PcieLocation {
    segment_group: PcieSegmentGroupNum,
    bus_segment:   PcieBusSegmentNum,
    device:        PcieDeviceNum,
    function:      PcieFunctionNum,
}

impl PcieLocation {
    const BUS_SEGMENT_SHIFT: usize = 20;
    /* Each bus occupies 1 MiB of ECAM address space */
    const DEVICE_SHIFT: usize = 15;
    /* Each device occupies 32 KiB of ECAM address space */
    const FUNCTION_SHIFT: usize = 12;

    pub fn new(
        segment_group: PcieSegmentGroupNum,
        bus_segment: PcieBusSegmentNum,
        device: PcieDeviceNum,
        function: PcieFunctionNum,
    ) -> Self {
        PcieLocation {
            segment_group,
            bus_segment,
            device,
            function,
        }
    }

    /* Each function occupies 4 KiB of ECAM address space */
    pub fn get_ecam_offset(&self) -> usize {
        let bus_offset = (self.bus_segment as usize) << Self::BUS_SEGMENT_SHIFT; /* Each bus occupies 1 MiB of ECAM address space */
        let device_offset = (self.device.get_inner() as usize) << Self::DEVICE_SHIFT; /* Each device occupies 32 KiB of ECAM address space */
        let function_offset = (self.function.get_inner() as usize) << Self::FUNCTION_SHIFT; /* Each function occupies 4 KiB of ECAM address space */
        bus_offset + device_offset + function_offset
    }
}

#[derive(Debug)]
pub struct PcieTopology {
    segments: Vec<PcieSegmentGroup>,
}

impl PcieTopology {
    pub fn new(segments: Vec<PcieSegmentGroup>) -> Self {
        PcieTopology {
            segments,
        }
    }

    pub(super) fn get_cfg_space_vaddr(
        &self,
        segment_group: PcieSegmentGroupNum,
        bus_segment: PcieBusSegmentNum,
        device_num: PcieDeviceNum,
        function_num: PcieFunctionNum,
    ) -> Result<VirtualAddress, Error> {
        let segment_group = self
            .segments
            .iter()
            .find(|sg| sg.pcie_segment_group_num == segment_group)
            .ok_or(Error::InvalidLocation)?;
        if bus_segment < segment_group.start_bus_num || bus_segment > segment_group.end_bus_num {
            return Err(Error::InvalidLocation);
        }
        let location = PcieLocation::new(
            segment_group.pcie_segment_group_num,
            bus_segment,
            device_num,
            function_num,
        );
        Ok(segment_group.ecam_vaddr + location.get_ecam_offset())
    }
}

#[derive(Debug)]
pub struct PcieSegmentGroup {
    pcie_segment_group_num: PcieSegmentGroupNum,
    ecam_vaddr:             VirtualAddress, /* Virtual address where this segment's ECAM is
                                             * mapped in the
                                             * kernel's address space */
    start_bus_num:          PcieBusSegmentNum,
    end_bus_num:            PcieBusSegmentNum,
    root_bus:               Box<PcieBusSegment>, /* Root bus of this segment's topology; the
                                                  * rest of the
                                                  * topology
                                                  * can be traversed from here */
}

impl PcieSegmentGroup {
    pub fn new(
        pcie_segment_group_num: PcieSegmentGroupNum,
        ecam_paddr: PhysicalAddress,
        start_bus_num: PcieBusSegmentNum,
        end_bus_num: PcieBusSegmentNum,
    ) -> Self {
        let ecam_vaddr = ecam::map_ecam(ecam_paddr);
        PcieSegmentGroup {
            pcie_segment_group_num,
            ecam_vaddr,
            start_bus_num,
            end_bus_num,
            root_bus: Box::new(PcieBusSegment::new(
                ecam_vaddr,
                pcie_segment_group_num,
                start_bus_num,
            )),
        }
    }
}

#[derive(Debug)]
pub struct PcieBusSegment {
    number:  PcieBusSegmentNum,
    devices: [PcieDevice; MAX_DEVICES_PER_BUS],
}

impl PcieBusSegment {
    fn new(
        ecam_vaddr: VirtualAddress,
        segment_group_num: PcieSegmentGroupNum,
        bus_num: PcieBusSegmentNum,
    ) -> Self {
        logln!(
            "[drivers::busses::pci_express] Enumerating PCIe bus segment {} of segment group {}",
            bus_num,
            segment_group_num
        );
        let mut devices: [PcieDevice; MAX_DEVICES_PER_BUS] =
            [const { PcieDevice::Empty }; MAX_DEVICES_PER_BUS];
        logln!(
            "[drivers::busses::pci_express] Initialized device array for bus segment {} of \
             segment group {}. Starting device enumeration...",
            bus_num,
            segment_group_num
        );
        for i in 0..MAX_DEVICES_PER_BUS {
            devices[i] = PcieDevice::new(ecam_vaddr, segment_group_num, bus_num, i as u8);
        }

        PcieBusSegment {
            number: bus_num,
            devices,
        }
    }
}

#[derive(Debug)]
pub enum PcieDevice {
    Empty,
    SingleFunc(PcieSingleFuncDevice),
    MultiFunc(PcieMultiFuncDevice),
}

impl PcieDevice {
    fn new(
        ecam_vaddr: VirtualAddress,
        segment_group_num: PcieSegmentGroupNum,
        bus_num: PcieBusSegmentNum,
        device_num: u8,
    ) -> Self {
        let cfg_space_vaddr = ecam_vaddr
            + PcieLocation::new(
                segment_group_num,
                bus_num,
                PcieDeviceNum(device_num),
                PcieFunctionNum(0),
            )
            .get_ecam_offset();

        let cfg_space = unsafe { &*cfg_space_vaddr.into_ptr::<PcieCfgSpace>() };
        if !cfg_space.has_device_present() {
            PcieDevice::Empty
        } else if cfg_space.device_is_multifunction() {
            PcieDevice::MultiFunc(PcieMultiFuncDevice::new(
                ecam_vaddr,
                segment_group_num,
                bus_num,
                device_num,
            ))
        } else {
            PcieDevice::SingleFunc(PcieSingleFuncDevice::new(
                ecam_vaddr,
                segment_group_num,
                bus_num,
                device_num,
            ))
        }
    }
}

#[derive(Debug)]
pub struct PcieSingleFuncDevice {
    number:   PcieDeviceNum,
    function: PcieFunction,
}

impl PcieSingleFuncDevice {
    fn new(
        ecam_vaddr: VirtualAddress,
        segment_group_num: PcieSegmentGroupNum,
        bus_num: PcieBusSegmentNum,
        device_num: u8,
    ) -> Self {
        PcieSingleFuncDevice {
            number:   PcieDeviceNum(device_num),
            function: PcieFunction::new(
                ecam_vaddr,
                segment_group_num,
                bus_num,
                device_num,
                PcieFunctionNum(0),
            ),
        }
    }
}

#[derive(Debug)]
pub struct PcieMultiFuncDevice {
    number:    PcieDeviceNum,
    functions: [PcieFunction; MAX_FUNCTIONS_PER_DEVICE],
}

impl PcieMultiFuncDevice {
    fn new(
        ecam_vaddr: VirtualAddress,
        segment_group_num: PcieSegmentGroupNum,
        bus_num: PcieBusSegmentNum,
        device_num: u8,
    ) -> Self {
        let mut functions: [PcieFunction; MAX_FUNCTIONS_PER_DEVICE] =
            [const { PcieFunction::Empty }; MAX_FUNCTIONS_PER_DEVICE];
        for i in 0..MAX_FUNCTIONS_PER_DEVICE {
            functions[i] = PcieFunction::new(
                ecam_vaddr,
                segment_group_num,
                bus_num,
                device_num,
                PcieFunctionNum::try_from(i as u8).unwrap(),
            );
        }

        PcieMultiFuncDevice {
            number: PcieDeviceNum(device_num),
            functions,
        }
    }
}

/* Number of 32-bit BARs */
const MAX_BAR_NUM: usize = 6;
/* Number of 64-bit BARs */
const MAX_EXT_BARS: usize = 3;

#[derive(Clone, Copy)]
union BarIoAddrs {
    pub bar32: [Option<VirtualAddress>; MAX_BAR_NUM],
    pub bar64: [Option<VirtualAddress>; MAX_EXT_BARS],
}

impl core::fmt::Debug for BarIoAddrs {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { write!(f, "BarIoAddrs {{ bar32: {:?}, bar64: {:?} }}", self.bar32, self.bar64) }
    }
}

#[derive(Debug)]
pub enum PcieFunction {
    Empty,
    Endpoint(Box<PcieEndpoint>), /* If this function is a normal endpoint device, then it has
                                  * no bus segment behind it and
                                  * can be represented as an endpoint struct containing its
                                  * relevant config space info
                                  * and BAR addresses */
    Bridge(Box<PcieBusSegment>), /* If this function is a bridge, then it has a bus segment
                                  * behind it which can be
                                  * traversed like the root bus segments in the
                                  * topology */
}

impl PcieFunction {
    fn new(
        ecam_vaddr: VirtualAddress,
        segment_group_num: PcieSegmentGroupNum,
        bus_num: PcieBusSegmentNum,
        device_num: u8,
        function_num: PcieFunctionNum,
    ) -> Self {
        let cfg_space_vaddr = ecam_vaddr
            + PcieLocation::new(
                segment_group_num,
                bus_num,
                PcieDeviceNum(device_num),
                function_num,
            )
            .get_ecam_offset();
        let cfg_space = unsafe { &*(cfg_space_vaddr.into_ptr::<PcieCfgSpace>()) };
        if !cfg_space.has_device_present() {
            PcieFunction::Empty
        } else if cfg_space.device_is_bridge() {
            let secondary_bus_segment_number =
                unsafe { cfg_space.header.bridge.get_secondary_bus_num() };
            PcieFunction::Bridge(Box::new(PcieBusSegment::new(
                ecam_vaddr,
                segment_group_num,
                secondary_bus_segment_number,
            )))
        } else {
            PcieFunction::Endpoint(Box::new(PcieEndpoint::new(
                ecam_vaddr,
                segment_group_num,
                bus_num,
                device_num,
                function_num,
            )))
        }
    }
}

#[derive(Debug)]
pub struct PcieEndpoint {
    number:     PcieFunctionNum,
    identifier: PciIdentifier,
    /* Raw pointer to this function's configuration space in the kernel's address space;
     * used for reading/writing config space registers inside this PCIe bus driver ONLY
     * other drivers and the rest of the kernel should use safe functions exposed by this bus
     * driver */
    cfg_ptr:    SpinMutex<NonNull<PcieCfgSpace>>,
}

impl PcieEndpoint {
    fn new(
        ecam_vaddr: VirtualAddress,
        segment_group_num: PcieSegmentGroupNum,
        bus_num: PcieBusSegmentNum,
        device_num: u8,
        function_num: PcieFunctionNum,
    ) -> Self {
        let cfg_space_vaddr = ecam_vaddr
            + PcieLocation::new(
                segment_group_num,
                bus_num,
                PcieDeviceNum(device_num),
                function_num,
            )
            .get_ecam_offset();
        let cfg_space = *&cfg_space_vaddr.into_ptr::<PcieCfgSpace>();
        let identifier = unsafe { (*cfg_space).header.common.get_identifier() };

        PcieEndpoint {
            number: function_num,
            identifier,
            cfg_ptr: SpinMutex::new(
                NonNull::new(cfg_space_vaddr.into_mut())
                    .expect("Invalid PCIe config space pointer"),
            ),
        }
    }
}

unsafe impl Send for PcieEndpoint {}
unsafe impl Sync for PcieEndpoint {}

/// Number of spaces each level of the topology tree is indented by when rendered for logging.
const TREE_INDENT_STEP: usize = 2;

impl core::fmt::Display for PcieTopology {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.segments.is_empty() {
            return write!(f, "  (no PCIe segment groups)");
        }
        for segment in &self.segments {
            segment.fmt_tree(f, TREE_INDENT_STEP)?;
        }
        Ok(())
    }
}

impl PcieSegmentGroup {
    fn fmt_tree(&self, f: &mut core::fmt::Formatter<'_>, indent: usize) -> core::fmt::Result {
        let ecam: u64 = self.ecam_vaddr.into();
        writeln!(
            f,
            "{:indent$}Segment Group {} (ECAM @ {:#018x}, buses {:#04x}-{:#04x})",
            "",
            self.pcie_segment_group_num,
            ecam,
            self.start_bus_num,
            self.end_bus_num,
            indent = indent
        )?;
        self.root_bus.fmt_tree(f, indent + TREE_INDENT_STEP)
    }
}

impl PcieBusSegment {
    fn fmt_tree(&self, f: &mut core::fmt::Formatter<'_>, indent: usize) -> core::fmt::Result {
        writeln!(f, "{:indent$}Bus {:#04x}", "", self.number, indent = indent)?;
        let child_indent = indent + TREE_INDENT_STEP;
        // Label the columns directly above the rows they describe (column widths must match the
        // formatting in `PcieFunction::fmt_tree`). Skipped for buses with no occupied slots.
        if self.devices.iter().any(|device| !matches!(device, PcieDevice::Empty)) {
            writeln!(
                f,
                "{:indent$}{:<7}  {:<9}  {}",
                "",
                "B:D.F",
                "VID:DID",
                "Class (cc:sc:pi)",
                indent = child_indent
            )?;
        }
        for device in &self.devices {
            match device {
                PcieDevice::Empty => {}
                PcieDevice::SingleFunc(dev) => {
                    dev.function.fmt_tree(
                        f,
                        child_indent,
                        self.number,
                        dev.number.get_inner(),
                        0,
                    )?;
                }
                PcieDevice::MultiFunc(dev) => {
                    for (func_num, function) in dev.functions.iter().enumerate() {
                        function.fmt_tree(
                            f,
                            child_indent,
                            self.number,
                            dev.number.get_inner(),
                            func_num as u8,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl PcieFunction {
    /// Renders a single function as one line of the topology tree, prefixed with its
    /// `bus:device.function` (BDF) address. Bridges additionally recurse into the bus segment
    /// behind them.
    fn fmt_tree(
        &self,
        f: &mut core::fmt::Formatter<'_>,
        indent: usize,
        bus: PcieBusSegmentNum,
        device: u8,
        function: u8,
    ) -> core::fmt::Result {
        match self {
            PcieFunction::Empty => Ok(()),
            PcieFunction::Endpoint(endpoint) => writeln!(
                f,
                "{:indent$}{:02x}:{:02x}.{:x}  {}",
                "",
                bus,
                device,
                function,
                endpoint.identifier,
                indent = indent
            ),
            PcieFunction::Bridge(secondary_bus) => {
                writeln!(
                    f,
                    "{:indent$}{:02x}:{:02x}.{:x}  PCI-to-PCI bridge -> bus {:#04x}",
                    "",
                    bus,
                    device,
                    function,
                    secondary_bus.number,
                    indent = indent
                )?;
                secondary_bus.fmt_tree(f, indent + TREE_INDENT_STEP)
            }
        }
    }
}
