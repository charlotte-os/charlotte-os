# The Charlotte Operating System (CharlotteOS)

---

## Programming Languages

- CharlotteOS is written primarily in the latest Edition of Rust, with architecture-specific assembly where required or advantageous.
- x86-64 assembly uses Intel syntax as implemented by `rustc`/`llvm-mc`.

---

## Platform & Firmware Requirements

CharlotteOS aims to support platforms that offer **standardized, documented, and interoperable hardware and firmware interfaces**. The focus is on systems where the operating system can rely on well-defined firmware and discoverability mechanisms, without requiring vendor-specific hacks or opaque initialization sequences.

### Supported Architectures and Their Requirements

#### x86-64 (Primary ISA)

- Invariant Timestamp Counter
- Local APIC with x2APIC mode
- Full standards conforming UEFI and ACPI firmware environment
- Intel or AMD compatible IOMMU

#### Aarch64 (Secondary ISA)

- ARMv8.2A or later
- GICv3 or later
- Secure Monitor Call (SMC) interface with PSCI
- SystemReady compliant firmware - Full or DT band

#### RISC-V (Tertiary ISA)

- RVA22 or later
- Advanced Interrupt Architecture (AIA)
- Supervisor Binary Interface (SBI)
- Boot and Runtime Services (BRS) specification compliant firmware

---

## Firmware Model

The Catten kernel requires at least the EBBR subset of UEFI and either of ACPI or Devicetree. The format is not the determining factor—**device documentation and correctness to the respective specification are.**

### UEFI

- Required for PC/server-class systems on all architectures.
- EBBR subset acceptable for embedded systems.

### ACPI

- Expected on PC/server-class machines across ISAs and all x86-64 systems.
- ACPI tables must be complete and spec-compliant enough to allow device discovery without vendor-specific workarounds or drivers.
- ACPI Machine Language (AML) code must be strictly specification conforming.

### Flattened Devicetree (FDT)

- Fully supported for SoC-style platforms.
- FDT must conform to the Devicetree specificion and accurately describe hardware resources.
- All `compatible` strings must map to publicly documented hardware blocks or IP cores or the described
hardware likely will not work.

### Documentation Requirement

Whether via ACPI or DT:

- Devices must be identifiable.
- Devices must be documented.
- “Unknown peripheral at address 0xXXXX” is not acceptable without vendor documentation.

This ensures that Catten can operate without relying on undocumented Linux driver behavior, hard-coded quirks, or vendor-specific hacks.

---

## Supported Hardware

### Memory[^1]

Embedded:

- Recommended: ≥ 128 MiB
- Minimum: 24 MiB

PC and Server:

- Recommended: ≥ 2 GiB
- Minimum: 256 MiB

### Storage[^1]

- Recommended: ≥ 64 GiB
- Minimum: 4 GiB
- Supported device classes:
  - NVMe (PCIe)
  - USB Mass Storage Device Class (MSC)
  - AHCI (SATA)
  - SCSI over PCIe serial

### Display

- Linear framebuffer exposed via:
  - UEFI GOP
  - FDT `simplefb` node

### Input Devices

- Keyboards:
  - i8042 PS/2
  - USB HID
  - I²C HID (documented ACPI/FDT only)
- Pointing Devices:
  - i8042 PS/2
  - USB HID
  - I²C HID (documented ACPI/FDT only)

### Serial Console

- NS16550 compatible UART
- Arm PL011 compatible UART
- USB CDC-ACM (virtual serial)

### Networking

- USB CDC-NCM (Ethernet over USB)

---

## Contributing

We welcome contributions of all forms—code, design proposals, documentation, and testing.  
Please join our Discord or Matrix communities if you’d like to get involved.

Community contributions for new hardware support will only be accepted when they include inline documentation comments with references to publicly available hardware documentation which may include community reverse engineered documentation along with clean, maintainable code.

---

## Licensing

The Charlotte Operating System is licensed under the GNU Affero General Public License version 3.0 (or any later version). By contributing, you agree that your work may be distributed under the AGPL version 3.0 or later.

---

## Community

Find us on:

- **Discord:** <https://discord.gg/vE7bCCKx4X>  
- **Matrix:** <https://matrix.to/#/#charlotteos:matrix.org>
- **Reddit** <https://www.reddit.com/r/charlotteos>
- **E-Mail** <charlotte-os@outlook.com>

[^1]: These requirements are estimates that may change in the course of development.
