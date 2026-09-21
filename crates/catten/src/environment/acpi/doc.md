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
SystemIO access, interrupt installation and deferred work. The x86-64 startup thread initializes
uACPI before enumerating PCI devices, and checks the result of every initialization stage, including
interrupt-model selection and GPE enablement.

The hand written `sdt` submodule continues to do the early table parsing the kernel needs before
an interpreter can reasonably run, and it will generally assume that target system firmware
appropriately conforms to the latest published ACPI specification or a prior forward compatible
version.


## Host services and execution order

The bootstrap processor starts a platform-initialization kernel thread after all local schedulers
and interrupt controllers are online. That thread prepares two persistent workers on LP 0,
initializes uACPI, loads and initializes the namespace, selects the interrupt model, and finalizes
GPE initialization. Only then is the namespace published as ready and peripheral discovery begun.
The early, hand-written SDT readers still supply MADT and MCFG data without depending on AML.

Host mutexes and counting events use scheduler waits with atomic registration against signals and
timer deadlines. A wakeup resumes the saved context on the same processor. Millisecond sleeps
actually relinquish the CPU, microsecond stalls busy-wait, and time reporting retains sub-second
precision. Interrupt flags are restored to their prior state even for nested operations.

On x86-64, the SCI is delivered through the IOAPIC using MADT overrides and ACPI's low/level defaults.
Dynamic IDT gates save registers and maintain interrupt depth. Uninstall masks the source, removes
the callback, and waits for active invocations before freeing its context. Deferred GPE and notify
callbacks have separate bounded queues (256 entries each), execute in thread context on LP 0, and
report queue exhaustion. Work completion waits for interrupt callbacks first, then queued and
executing work. Call it from ordinary thread context, never from a worker waiting for itself.

PCI access requires MCFG ECAM independently of device enumeration. Functions outside the
firmware ECAM allocations return NOT_FOUND; there is no legacy configuration-port fallback.
Legacy PCI devices are supported only behind PCIe bridges using ECAM configuration access. Access widths, alignment, and byte offsets
are checked. SystemIO mappings may overlap, and each access must fit completely within its view.

Physical mappings preserve page offsets, reject overflowing ranges, roll back failed mappings, and
select an uncached PAT type for MMIO. The kernel does not yet have synchronous cross-CPU TLB
shootdown. Consequently, ACPI owns a separate virtual aperture: unmapping clears the leaf but retains
its paging structures, and retired virtual addresses are never reused. On a 48-bit machine the
aperture is 1 TiB; exhaustion is reported as a mapping failure. Physical firmware memory is never
freed by unmap. Reclaiming retired virtual addresses and paging structures requires a future kernel
TLB-shootdown implementation. Completed kernel threads similarly park with their stack retained
until a general thread reaper exists; ACPI uses persistent workers rather than a thread per event.

AArch64 and RISC-V builds compile the same host ABI, but their existing interrupt-controller,
scheduler-context, and timer implementations are still incomplete architecture ports. Full runtime
integration is currently supported on x86-64. Unsupported SystemIO or interrupt routing on other
architectures returns an explicit status rather than pretending to install hardware services.

## Verification

Build and boot the host-service self tests with:

```sh
python3 scripts/test-uacpi.py
```

The script needs QEMU, mtools, and OVMF; use `--firmware` to override the firmware path. It uses TCG,
temporary FAT images, and no privileged mounts. The default run tests 1, 2, and 8 CPUs on the PCIe q35 machine. TCG is configured for the kernel's
current four-level paging implementation. Serial transcripts go to `logs/uacpi-smoke-*.log`.

The `acpi_self_test` feature checks allocation/alignment failures, mapping offsets and overflow,
PCI offsets and bounds, overlapping SystemIO views, interrupt-flag nesting, timer precision,
counting events and timeouts, mutex contention across CPUs, worker affinity, work draining, and
namespace evaluation. The script then injects two power-button interrupts and checks that both
reach uACPI and wake a deferred worker after the system has become idle. The power-button handler
is a test fixture enabled only by this feature.
