use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::{env, fs};

/// Vendors whose individual device names are always compiled into the kernel. Device names for
/// every other vendor are only included when the `pci_ids_full` feature is enabled since the full
/// database adds roughly a megabyte to the kernel image. Vendor, class, subclass and programming
/// interface names are always included in full as they are comparatively tiny.
const KEY_PCI_VENDORS: &[u16] = &[
    0x8086, // Intel Corporation
    0x1022, // Advanced Micro Devices, Inc. [AMD]
    0x1002, // Advanced Micro Devices, Inc. [AMD/ATI]
    0x10de, // NVIDIA Corporation
    0x1af4, // Red Hat, Inc. (VirtIO)
    0x1b36, // Red Hat, Inc. (QEMU)
    0x13b5, // ARM
    0x17cb, // Qualcomm Technologies, Inc
    0x168c, // Qualcomm Atheros
    0x14e4, // Broadcom Inc. and subsidiaries
    0x1166, // Broadcom (formerly ServerWorks)
    0x1000, // Broadcom / LSI
    0x10ec, // Realtek Semiconductor Co., Ltd.
    0x144d, // Samsung Electronics Co Ltd
    0x11ab, // Marvell Technology Group Ltd.
    0x1b4b, // Marvell Technology Group Ltd.
    0x15ad, // VMware
    0x1414, // Microsoft Corporation (Hyper-V)
    0x5853, // XenSource, Inc.
    0x80ee, // InnoTek Systemberatung GmbH (VirtualBox)
    0x1ab8, // Parallels, Inc.
    0x1d0f, // Amazon.com, Inc.
    0x1ae0, // Google, Inc.
];

/// Names are stored back to back in a single string pool and referenced by offset and length
/// from tables sorted by their (packed) numeric IDs.
#[derive(Default)]
struct PciIds {
    pool: String,
    vendors: BTreeMap<u32, (usize, usize)>,
    devices: BTreeMap<u32, (usize, usize)>,
    classes: BTreeMap<u32, (usize, usize)>,
    subclasses: BTreeMap<u32, (usize, usize)>,
    prog_ifs: BTreeMap<u32, (usize, usize)>,
}

impl PciIds {
    fn intern(&mut self, name: &str) -> (usize, usize) {
        let start = self.pool.len();
        self.pool.push_str(name);
        (start, name.len())
    }

    /// Parses the `pci.ids` format: vendors and classes start at column zero, their devices and
    /// subclasses are indented by one tab and subsystems and programming interfaces by two.
    fn parse(src: &str, all_devices: bool) -> Self {
        let mut ids = Self::default();
        let mut in_classes = false;
        let mut vendor: Option<u16> = None;
        let mut class: Option<u8> = None;
        let mut subclass: Option<u8> = None;
        for line in src.lines() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(rest) = line.strip_prefix("C ") {
                in_classes = true;
                let (id, name) = split_id(rest);
                let id = u8::from_str_radix(id, 16).expect("invalid PCI class code");
                class = Some(id);
                subclass = None;
                let name = ids.intern(name);
                ids.classes.entry(id as u32).or_insert(name);
                continue;
            }
            let depth = line.bytes().take_while(|&b| b == b'\t').count();
            let (id, name) = split_id(&line[depth..]);
            if in_classes {
                let class = class.expect("PCI subclass outside of a class");
                match depth {
                    1 => {
                        let id = u8::from_str_radix(id, 16).expect("invalid PCI subclass");
                        subclass = Some(id);
                        let name = ids.intern(name);
                        ids.subclasses.entry(((class as u32) << 8) | id as u32).or_insert(name);
                    }
                    2 => {
                        let subclass = subclass.expect("PCI prog IF outside of a subclass");
                        let id = u8::from_str_radix(id, 16).expect("invalid PCI prog IF");
                        let key = ((class as u32) << 16) | ((subclass as u32) << 8) | id as u32;
                        let name = ids.intern(name);
                        ids.prog_ifs.entry(key).or_insert(name);
                    }
                    _ => panic!("unexpected indentation in the pci.ids class section: {line:?}"),
                }
            } else {
                match depth {
                    0 => {
                        let id = u16::from_str_radix(id, 16).expect("invalid PCI vendor ID");
                        vendor = Some(id);
                        let name = ids.intern(name);
                        ids.vendors.entry(id as u32).or_insert(name);
                    }
                    1 => {
                        let vendor = vendor.expect("PCI device outside of a vendor");
                        if all_devices || KEY_PCI_VENDORS.contains(&vendor) {
                            let id = u16::from_str_radix(id, 16).expect("invalid PCI device ID");
                            let name = ids.intern(name);
                            ids.devices.entry(((vendor as u32) << 16) | id as u32).or_insert(name);
                        }
                    }
                    // Subsystem IDs are not used by the kernel.
                    _ => {}
                }
            }
        }
        ids
    }

    fn emit_table(out: &mut String, name: &str, table: &BTreeMap<u32, (usize, usize)>) {
        writeln!(out, "pub(super) static {name}: [PciIdName; {}] = [", table.len()).unwrap();
        for (key, (start, len)) in table {
            writeln!(out, "    PciIdName::new({key:#x}, {start}, {len}),").unwrap();
        }
        out.push_str("];\n");
    }

    fn generate(&self) -> String {
        let mut out = String::new();
        out.push_str("// @generated by build.rs from vendor/pci-ids/pci.ids. Do not edit.\n");
        writeln!(out, "pub(super) static NAME_POOL: &str = {:?};", self.pool).unwrap();
        Self::emit_table(&mut out, "VENDORS", &self.vendors);
        Self::emit_table(&mut out, "DEVICES", &self.devices);
        Self::emit_table(&mut out, "CLASSES", &self.classes);
        Self::emit_table(&mut out, "SUBCLASSES", &self.subclasses);
        Self::emit_table(&mut out, "PROG_IFS", &self.prog_ifs);
        out
    }
}

fn split_id(s: &str) -> (&str, &str) {
    let (id, name) = s.split_once(char::is_whitespace).unwrap_or((s, ""));
    (id, name.trim())
}

fn generate_pci_ids() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_path = manifest_dir.join("vendor/pci-ids/pci.ids");
    println!("cargo:rerun-if-changed={}", src_path.display());
    let raw = fs::read(&src_path).expect("failed to read vendor/pci-ids/pci.ids");
    let src = String::from_utf8_lossy(&raw);
    let all_devices = env::var_os("CARGO_FEATURE_PCI_IDS_FULL").is_some();
    let generated = PciIds::parse(&src, all_devices).generate();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("pci_ids.rs");
    fs::write(out_path, generated).expect("failed to write the generated PCI ID tables");
}

fn main() {
    generate_pci_ids();

    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    match arch.as_str() {
        "x86_64" => {
            // Tell cargo to pass the linker script to the linker...
            println!("cargo:rustc-link-arg=-T./crates/catten/linker/x86_64.ld");
            // ...and to re-run if it changes.
            println!("cargo:rerun-if-changed=./crates/catten/linker/x86_64.ld");
        }
        "aarch64" => {
            // Tell cargo to pass the linker script to the linker...
            println!("cargo:rustc-link-arg=-T./crates/catten/linker/aarch64.ld");
            // ...and to re-run if it changes.
            println!("cargo:rerun-if-changed=./crates/catten/linker/aarch64.ld");
        }
        "riscv64" => {
            // Tell cargo to pass the linker script to the linker...
            println!("cargo:rustc-link-arg=-T./crates/catten/linker/riscv64.ld");
            // ...and to re-run if it changes.
            println!("cargo:rerun-if-changed=./crates/catten/linker/riscv64.ld");
        }
        _ => panic!("Invalid ISA"),
    }

    println!("cargo:rerun-if-changed=asm");
}
