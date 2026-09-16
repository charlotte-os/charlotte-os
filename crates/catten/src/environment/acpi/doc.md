The Advanced Configuration and Power Interface (ACPI) Subsystem

The Advanced Configuration and Power Interface (ACPI) is an open industry specification that
defines a flexible and extensible interface for hardware discovery, configuration, power
management, and monitoring. ACPI provides a standardized way for the operating system to
interact with the underlying hardware, allowing it to manage power states, configure devices,
and perform other system-level tasks in a platform-independent manner.

It provides information in two different forms:

- System Description Tables (SDTs)
- ACPI Machine Language (AML) bytecode tables

This module is split into two submodules, `sdt` and `aml`, which contain code for working with
each of these forms of information respectively and their inline documentation contains more
detailed information about how each of them works and how Catten uses them.

This top level module contains code for finding and parsing the XSDT to find the physical
addresses of other ACPI tables, as well as some common data structures and utilities for working
with the headers that are common to all ACPI tables.

The primary reference needed to understand and work with this module is the [ACPI specification](https://uefi.org/specs/ACPI/6.6/).
In addition to the specification the source code of [uACPI](https://github.com/uACPI/uACPI) a portable C language
ACPI implementation made to be robust enough to handle buggy firmware can be very helpful to
understand how to work with ACPI and handle various edge cases and quirks of real world
firmware.

Catten now vendors uACPI and links it through the `uacpi-wrapper` crate, which compiles it for
the kernel's target and generates raw bindings to its public headers. uACPI is only half a
library: everything it declares in `uacpi/kernel_api.h` is a function the kernel owes it, and the
`uacpi_host` submodule here is where Catten answers for memory, mapping, locking, timing, PCI and
SystemIO access, interrupt installation and deferred work. That module is scaffolding at present,
so nothing has driven the interpreter yet.

The hand written `sdt` submodule continues to do the early table parsing the kernel needs before
an interpreter can reasonably run, and it will generally assume that target system firmware
appropriately conforms to the latest published ACPI specification or a prior forward compatible
version.
