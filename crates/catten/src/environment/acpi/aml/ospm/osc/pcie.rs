use core::ffi::{c_char, c_void};
use core::ptr;

use hashbrown::HashMap;
use spin::LazyLock;
use uacpi_wrapper::*;

use crate::cpu::scheduler::sync::rwlock::RwLock;
use crate::device_management::drivers::busses::pci_express::topology::PcieSegmentGroupNum;
use crate::logln;

pub static PCIE_OSC_RESULTS: LazyLock<RwLock<HashMap<PcieSegmentGroupNum, u32>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// 33DB4D5B-1FF7-401C-9657-7441C03DD766 in ToUUID byte order
/// (first three fields little-endian, last two as-is).
const PCI_OSC_UUID: [u8; 16] = [
    0x5b, 0x4d, 0xdb, 0x33, 0xf7, 0x1f, 0x1c, 0x40, 0x96, 0x57, 0x74, 0x41, 0xc0, 0x3d, 0xd7, 0x66,
];

const OSC_QUERY: u32 = 1 << 0;
/// Status DWORD bits 1..=3: _OSC failure, bad UUID, bad revision.
/// Bit 4 (capabilities masked) is informational, not an error.
const OSC_ERR_MASK: u32 = 0b1110;

pub mod support {
    pub const EXT_CONFIG: u32 = 1 << 0;
    pub const ASPM: u32 = 1 << 1;
    pub const CLOCK_PM: u32 = 1 << 2;
    pub const SEGMENT_GROUPS: u32 = 1 << 3;
    pub const MSI: u32 = 1 << 4;
}

pub mod control {
    pub const PCIE_NATIVE_HOTPLUG: u32 = 1 << 0;
    pub const SHPC_NATIVE_HOTPLUG: u32 = 1 << 1;
    pub const PME: u32 = 1 << 2;
    pub const AER: u32 = 1 << 3;
    pub const PCIE_CAP_STRUCTURE: u32 = 1 << 4;
    pub const LTR: u32 = 1 << 5;
    pub const DPC: u32 = 1 << 7;
}

#[derive(Debug)]
pub enum OscError {
    NoMemory,
    Eval(uacpi_status),
    BadReturn,
    Firmware(u32), // raw status DWORD
}

/// Owning handle; unrefs on drop.
struct Obj(*mut uacpi_object);

impl Obj {
    fn new(p: *mut uacpi_object) -> Result<Self, OscError> {
        if p.is_null() {
            Err(OscError::NoMemory)
        } else {
            Ok(Obj(p))
        }
    }
}

impl Drop for Obj {
    fn drop(&mut self) {
        unsafe { uacpi_object_unref(self.0) }
    }
}

fn buffer_obj(bytes: &[u8]) -> Result<Obj, OscError> {
    let mut view: uacpi_data_view = unsafe { core::mem::zeroed() };
    view.__bindgen_anon_1.const_bytes = bytes.as_ptr();
    view.length = bytes.len() as _;
    // uacpi_object_create_buffer copies the data, so `bytes` needn't outlive this.
    Obj::new(unsafe { uacpi_object_create_buffer(view) })
}

/// Evaluates the PCI host bridge _OSC. Returns the control mask firmware granted.
///
/// # Safety
/// `bridge` must be a valid namespace node for a PCI host bridge.
unsafe fn pci_osc(
    bridge: *mut uacpi_namespace_node,
    query: bool,
    support: u32,
    control: u32,
) -> Result<u32, OscError> {
    let mut caps = [0u8; 12];
    caps[0..4].copy_from_slice(
        &(if query {
            OSC_QUERY
        } else {
            0
        })
        .to_le_bytes(),
    );
    caps[4..8].copy_from_slice(&support.to_le_bytes());
    caps[8..12].copy_from_slice(&control.to_le_bytes());

    let uuid = buffer_obj(&PCI_OSC_UUID)?;
    let rev = Obj::new(unsafe { uacpi_object_create_integer(1) })?;
    let count = Obj::new(unsafe { uacpi_object_create_integer(3) })?;
    let caps_obj = buffer_obj(&caps)?;

    let mut argv = [uuid.0, rev.0, count.0, caps_obj.0];
    let args = uacpi_object_array {
        objects: argv.as_mut_ptr(),
        count: argv.len() as _,
    };

    let mut ret: *mut uacpi_object = ptr::null_mut();
    let st = unsafe {
        uacpi_eval_typed(bridge, c"_OSC".as_ptr(), &args, UACPI_OBJECT_BUFFER_BIT as _, &mut ret)
    };
    if st != UACPI_STATUS_OK {
        return Err(OscError::Eval(st)); // UACPI_STATUS_NOT_FOUND => no _OSC here
    }
    let ret = Obj::new(ret).map_err(|_| OscError::BadReturn)?;

    let mut out: uacpi_data_view = unsafe { core::mem::zeroed() };
    if unsafe { uacpi_object_get_buffer(ret.0, &mut out) } != UACPI_STATUS_OK {
        return Err(OscError::BadReturn);
    }
    let len = out.length as usize;
    if len < 12 {
        return Err(OscError::BadReturn);
    }
    let bytes = unsafe { core::slice::from_raw_parts(out.__bindgen_anon_1.const_bytes, len) };
    let dword = |i: usize| u32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());

    let status = dword(0);
    if status & OSC_ERR_MASK != 0 {
        return Err(OscError::Firmware(status));
    }
    Ok(dword(2))
}

unsafe extern "C" fn on_host_bridge(
    _user: *mut c_void,
    node: *mut uacpi_namespace_node,
    _depth: u32,
) -> uacpi_iteration_decision {
    let mut res_buf: u64 = 0;
    let _ = unsafe { uacpi_eval_simple_integer(node, c"_SEG".as_ptr(), &mut res_buf) };
    let seg_group = res_buf as PcieSegmentGroupNum;

    let support = support::EXT_CONFIG
        | support::ASPM
        | support::CLOCK_PM
        | support::SEGMENT_GROUPS
        | support::MSI;
    let want =
        control::PCIE_NATIVE_HOTPLUG | control::PME | control::AER | control::PCIE_CAP_STRUCTURE;

    let result = unsafe {
        pci_osc(node, true, support, want)
            .and_then(|offered| pci_osc(node, false, support, offered))
    };

    match result {
        Ok(granted) => {
            logln!(
                "[ACPI][OSPM] Successfully negotiated _OSC for host bridge with segment group \
                 {:?}. Granted features: 0b{:b}",
                seg_group,
                granted
            );
            PCIE_OSC_RESULTS.write().insert(seg_group, granted);
        }
        Err(OscError::Eval(UACPI_STATUS_NOT_FOUND)) => {
            logln!(
                "[ACPI][OSPM] No _OSC found for host bridge with segment group {:?}. Firmware \
                 retains control.",
                seg_group
            );
            PCIE_OSC_RESULTS.write().insert(seg_group, 0);
        }
        Err(_e) => {
            logln!(
                "[ACPI][OSPM] Failed to negotiate _OSC for host bridge with segment group {:?}. \
                 Leaving features firmware-owned.",
                seg_group
            );
            PCIE_OSC_RESULTS.write().insert(seg_group, 0);
        }
    }

    UACPI_ITERATION_DECISION_CONTINUE
}

pub fn negotiate_pci_osc() {
    let hids: [*const c_char; 3] = [c"PNP0A08".as_ptr(), c"PNP0A03".as_ptr(), ptr::null()];
    unsafe {
        let _ = uacpi_find_devices_at(
            uacpi_namespace_root(),
            hids.as_ptr(),
            Some(on_host_bridge),
            ptr::null_mut(),
        );
    }
}
