# The Charlotte Operating System (CharlotteOS)

---

## Programming Languages

- CharlotteOS is written primarily in the latest Edition of Rust, with architecture-specific assembly where required or advantageous.
- x86-64 assembly uses Intel syntax as implemented by `rustc`/`llvm-mc`.

---

## Platform & Firmware Requirements

CharlotteOS aims to support platforms that offer **standardized, documented, and interoperable hardware and firmware interfaces**. The focus is on systems where the operating system can rely on well-defined firmware and discoverability mechanisms, without requiring vendor-specific hacks or opaque initialization sequences.

### Supported Architectures and Their Requirements

#### x86-64

- Invariant Timestamp Counter
- Local APIC with x2APIC mode
- Full standards conforming UEFI and ACPI firmware environment
- Intel or AMD compatible IOMMU

#### *Other architectures may be supported in the future depending on contributor support and demand for their development.*

---

## Firmware Model

System firmware is required to implement the UEFI specification and version 2.0 or later of the ACPI specification.

The latest versions of both specifications can be found at <https://uefi.org/specifications>.

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
  - [Planned] NVMe (PCIe)
  - [Planned] USB Mass Storage Device Class (MSC)
  - [Planned] AHCI (SATA)
  - [Planned] SDHCI (PCIe SD card reader)

### Display

- Linear framebuffer exposed via UEFI GOP

### Input Devices

- Keyboards:
  - [Planned] i8042 PS/2
  - [Planned] USB HID
  - [Planned] I²C HID

- Pointing Devices:
  - [Planned] i8042 PS/2
  - [Planned] USB HID
  - [Planned] I²C HID

### Serial Console

- [Planned] NS16550 compatible UART over PCIe
- [Planned] USB CDC-ACM (virtual serial)

### Networking

- [Planned] USB CDC-NCM (Ethernet over USB)

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
