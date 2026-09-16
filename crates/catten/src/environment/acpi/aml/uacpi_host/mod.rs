//! # The Host Half of the uACPI Interface
//!
//! uACPI is only half a library. Everything declared in its `uacpi/kernel_api.h` is a function
//! that *the kernel owes uACPI*, and the [`uacpi-wrapper`](uacpi_wrapper) crate deliberately
//! supplies none of them: they are memory, mapping, locking, timing, PCI and SystemIO access,
//! interrupt installation and deferred work, all of which only Catten can answer for. This module
//! is where Catten answers.
//!
//! Every function here is exported unmangled under the exact name uACPI's C sources reference, so
//! the set below has to stay complete. Dropping one does not produce a compile error; it produces
//! an undefined reference at link time for the whole kernel.
//!
//! # Configuration
//!
//! These signatures match the configuration [`uacpi-wrapper`](uacpi_wrapper) compiles uACPI with,
//! which is the upstream default. Three of those defaults shape this list directly:
//!
//! - `UACPI_SIZED_FREES` is off, so [`uacpi_kernel_free`] gets no size hint and the allocator has
//!   to recover the layout itself.
//! - `UACPI_NATIVE_MMIO` is off, so uACPI uses its own volatile load and store helpers and no
//!   `uacpi_kernel_mmio_*` functions are owed.
//! - `UACPI_NATIVE_ALLOC_ZEROED` is off, so uACPI zeroes [`uacpi_kernel_alloc`]'s result itself and
//!   no `uacpi_kernel_alloc_zeroed` is owed.
//!
//! # Status
//!
//! This is scaffolding. Every body below is a [`todo!`], which means the kernel links but takes a
//! panic the moment uACPI reaches for anything. Nothing should call
//! [`uacpi_setup_early_table_access`](uacpi_wrapper::uacpi_setup_early_table_access) or
//! [`uacpi_initialize`](uacpi_wrapper::uacpi_initialize) until the functions the chosen
//! initialisation level depends on are real.

use uacpi_wrapper::{
    uacpi_bool,
    uacpi_cpu_flags,
    uacpi_firmware_request,
    uacpi_handle,
    uacpi_interrupt_handler,
    uacpi_interrupt_state,
    uacpi_io_addr,
    uacpi_log_level,
    uacpi_pci_address,
    uacpi_phys_addr,
    uacpi_size,
    uacpi_status,
    uacpi_thread_id,
    uacpi_u8,
    uacpi_u16,
    uacpi_u32,
    uacpi_u64,
    uacpi_work_handler,
    uacpi_work_type,
};

use crate::memory::PhysicalAddress;

/* ------------------------------------------------------------------------------------------- *
 * Table discovery                                                                              *
 * ------------------------------------------------------------------------------------------- */

/// Reports the physical address of the RSDP, which uACPI cannot find for itself because every boot
/// protocol hands it over differently.
///
/// Limine gives it to us directly, so this is the one entry point that has a real answer waiting.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_get_rsdp(
    out_rsdp_address: *mut uacpi_phys_addr,
) -> uacpi_status {
    unsafe {
        *out_rsdp_address =
            <PhysicalAddress as Into<u64>>::into(*crate::environment::acpi::RSDP_ADDR);
    }
    uacpi_wrapper::UACPI_STATUS_OK
}

/* ------------------------------------------------------------------------------------------- *
 * Virtual memory                                                                               *
 * ------------------------------------------------------------------------------------------- */

/// Maps `len` bytes of physical memory starting at `addr` and returns a virtual address for it.
///
/// uACPI expects this to succeed for anything the firmware points it at, including regions that
/// are not backed by conventional memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_map(
    addr: uacpi_phys_addr,
    len: uacpi_size,
) -> *mut core::ffi::c_void {
    let _ = (addr, len);
    todo!("Map the requested physical range, most likely straight through the HHDM.")
}

/// Releases a mapping previously returned by [`uacpi_kernel_map`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_unmap(addr: *mut core::ffi::c_void, len: uacpi_size) {
    let _ = (addr, len);
    todo!("Undo the corresponding uacpi_kernel_map.")
}

/* ------------------------------------------------------------------------------------------- *
 * Logging                                                                                      *
 * ------------------------------------------------------------------------------------------- */

/// Emits one already-formatted, NUL-terminated uACPI log message.
///
/// uACPI terminates each message with a bare `\n`, which is not what the kernel terminal wants; see
/// [`crate::log::flanterm`] on line endings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_log(
    level: uacpi_log_level,
    message: *const core::ffi::c_char,
) {
    let _ = (level, message);
    todo!("Forward the message to the kernel log, translating the level and the line ending.")
}

/* ------------------------------------------------------------------------------------------- *
 * PCI configuration space                                                                      *
 * ------------------------------------------------------------------------------------------- */

/// Opens a handle to the configuration space of one PCI device.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_pci_device_open(
    address: uacpi_pci_address,
    out_handle: *mut uacpi_handle,
) -> uacpi_status {
    let _ = (address, out_handle);
    todo!("Resolve the segment/bus/device/function to a PCIe configuration space handle.")
}

/// Closes a handle returned by [`uacpi_kernel_pci_device_open`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_pci_device_close(device: uacpi_handle) {
    let _ = device;
    todo!("Release the configuration space handle.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_pci_read8(
    device: uacpi_handle,
    offset: uacpi_size,
    value: *mut uacpi_u8,
) -> uacpi_status {
    let _ = (device, offset, value);
    todo!("Read one byte of PCI configuration space.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_pci_read16(
    device: uacpi_handle,
    offset: uacpi_size,
    value: *mut uacpi_u16,
) -> uacpi_status {
    let _ = (device, offset, value);
    todo!("Read one word of PCI configuration space.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_pci_read32(
    device: uacpi_handle,
    offset: uacpi_size,
    value: *mut uacpi_u32,
) -> uacpi_status {
    let _ = (device, offset, value);
    todo!("Read one dword of PCI configuration space.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_pci_write8(
    device: uacpi_handle,
    offset: uacpi_size,
    value: uacpi_u8,
) -> uacpi_status {
    let _ = (device, offset, value);
    todo!("Write one byte of PCI configuration space.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_pci_write16(
    device: uacpi_handle,
    offset: uacpi_size,
    value: uacpi_u16,
) -> uacpi_status {
    let _ = (device, offset, value);
    todo!("Write one word of PCI configuration space.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_pci_write32(
    device: uacpi_handle,
    offset: uacpi_size,
    value: uacpi_u32,
) -> uacpi_status {
    let _ = (device, offset, value);
    todo!("Write one dword of PCI configuration space.")
}

/* ------------------------------------------------------------------------------------------- *
 * The SystemIO address space                                                                   *
 * ------------------------------------------------------------------------------------------- */

/// Claims the SystemIO range `[base, base + len)` and returns a handle the accessors below index
/// into.
///
/// On x86_64 this address space is the port I/O space reached with `in` and `out`; elsewhere it has
/// to be synthesised or refused.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_io_map(
    base: uacpi_io_addr,
    len: uacpi_size,
    out_handle: *mut uacpi_handle,
) -> uacpi_status {
    let _ = (base, len, out_handle);
    todo!("Claim the SystemIO range and hand back a handle describing it.")
}

/// Releases a range claimed by [`uacpi_kernel_io_map`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_io_unmap(handle: uacpi_handle) {
    let _ = handle;
    todo!("Release the SystemIO range.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_io_read8(
    handle: uacpi_handle,
    offset: uacpi_size,
    out_value: *mut uacpi_u8,
) -> uacpi_status {
    let _ = (handle, offset, out_value);
    todo!("Read one byte from the mapped SystemIO range, as a single access of exactly that width.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_io_read16(
    handle: uacpi_handle,
    offset: uacpi_size,
    out_value: *mut uacpi_u16,
) -> uacpi_status {
    let _ = (handle, offset, out_value);
    todo!("Read one word from the mapped SystemIO range, as a single access of exactly that width.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_io_read32(
    handle: uacpi_handle,
    offset: uacpi_size,
    out_value: *mut uacpi_u32,
) -> uacpi_status {
    let _ = (handle, offset, out_value);
    todo!(
        "Read one dword from the mapped SystemIO range, as a single access of exactly that width."
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_io_write8(
    handle: uacpi_handle,
    offset: uacpi_size,
    in_value: uacpi_u8,
) -> uacpi_status {
    let _ = (handle, offset, in_value);
    todo!("Write one byte to the mapped SystemIO range, as a single access of exactly that width.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_io_write16(
    handle: uacpi_handle,
    offset: uacpi_size,
    in_value: uacpi_u16,
) -> uacpi_status {
    let _ = (handle, offset, in_value);
    todo!("Write one word to the mapped SystemIO range, as a single access of exactly that width.")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_io_write32(
    handle: uacpi_handle,
    offset: uacpi_size,
    in_value: uacpi_u32,
) -> uacpi_status {
    let _ = (handle, offset, in_value);
    todo!("Write one dword to the mapped SystemIO range, as a single access of exactly that width.")
}

/* ------------------------------------------------------------------------------------------- *
 * Allocation                                                                                   *
 * ------------------------------------------------------------------------------------------- */

/// Allocates `size` bytes, aligned well enough for any type uACPI puts in them.
///
/// The pair below is the only thing uACPI knows about memory, so whatever alignment this promises,
/// [`uacpi_kernel_free`] has to rebuild from the pointer alone: uACPI is compiled without
/// `UACPI_SIZED_FREES` and so passes no size back.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_alloc(size: uacpi_size) -> *mut core::ffi::c_void {
    let _ = size;
    todo!("Allocate from the kernel heap, recording whatever `free` will need to reverse it.")
}

/// Frees a pointer previously returned by [`uacpi_kernel_alloc`]. Null is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_free(mem: *mut core::ffi::c_void) {
    let _ = mem;
    todo!("Return the allocation to the kernel heap.")
}

/* ------------------------------------------------------------------------------------------- *
 * Time                                                                                         *
 * ------------------------------------------------------------------------------------------- */

/// Returns a monotonic nanosecond count since boot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_get_nanoseconds_since_boot() -> uacpi_u64 {
    todo!("Read the monotonic system timer.")
}

/// Busy-waits for at least `usec` microseconds without yielding.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_stall(usec: uacpi_u8) {
    let _ = usec;
    todo!("Spin for the requested number of microseconds.")
}

/// Sleeps for at least `msec` milliseconds, yielding if the kernel can.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_sleep(msec: uacpi_u64) {
    let _ = msec;
    todo!("Sleep for the requested number of milliseconds.")
}

/* ------------------------------------------------------------------------------------------- *
 * Mutexes and events                                                                           *
 * ------------------------------------------------------------------------------------------- */

/// Creates a mutex, which uACPI may block on and which must therefore not be used from interrupt
/// context.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_create_mutex() -> uacpi_handle {
    todo!("Allocate a kernel mutex and return an opaque handle to it.")
}

/// Destroys a mutex created by [`uacpi_kernel_create_mutex`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_free_mutex(handle: uacpi_handle) {
    let _ = handle;
    todo!("Destroy the mutex behind the handle.")
}

/// Acquires a mutex, where `timeout` is `0` for a single non-blocking attempt, `0xffff` for an
/// unbounded wait, and anything between for that many milliseconds.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_acquire_mutex(
    handle: uacpi_handle,
    timeout: uacpi_u16,
) -> uacpi_status {
    let _ = (handle, timeout);
    todo!("Acquire the mutex, honouring the timeout encoding described above.")
}

/// Releases a mutex acquired by [`uacpi_kernel_acquire_mutex`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_release_mutex(handle: uacpi_handle) {
    let _ = handle;
    todo!("Release the mutex.")
}

/// Creates a counting event, the primitive uACPI waits on for firmware completions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_create_event() -> uacpi_handle {
    todo!("Allocate an event object and return an opaque handle to it.")
}

/// Destroys an event created by [`uacpi_kernel_create_event`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_free_event(handle: uacpi_handle) {
    let _ = handle;
    todo!("Destroy the event behind the handle.")
}

/// Waits for an event to be signalled, with `timeout` in milliseconds or `0xffff` for an unbounded
/// wait. Returns false on timeout.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_wait_for_event(
    handle: uacpi_handle,
    timeout: uacpi_u16,
) -> uacpi_bool {
    let _ = (handle, timeout);
    todo!("Wait on the event and report whether it was signalled before the timeout.")
}

/// Signals an event, incrementing its counter and releasing one waiter.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_signal_event(handle: uacpi_handle) {
    let _ = handle;
    todo!("Signal the event.")
}

/// Resets an event's counter to zero without releasing anyone.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_reset_event(handle: uacpi_handle) {
    let _ = handle;
    todo!("Reset the event counter.")
}

/* ------------------------------------------------------------------------------------------- *
 * Execution context                                                                            *
 * ------------------------------------------------------------------------------------------- */

/// Identifies the calling thread. uACPI uses this for AML mutex ownership, so the value has to be
/// unique per thread and must never be `uacpi_thread_id::MAX`, which uACPI reserves as its "no
/// thread" sentinel.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_get_thread_id() -> uacpi_thread_id {
    todo!("Return a per-thread identifier that is never the all-ones sentinel.")
}

/// Disables interrupts on the calling LP and returns the prior state for
/// [`uacpi_kernel_restore_interrupts`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_disable_interrupts() -> uacpi_interrupt_state {
    todo!("Mask interrupts on this LP and return the previous state.")
}

/// Restores the interrupt state captured by [`uacpi_kernel_disable_interrupts`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_restore_interrupts(state: uacpi_interrupt_state) {
    let _ = state;
    todo!("Restore the interrupt state this LP had before it was masked.")
}

/* ------------------------------------------------------------------------------------------- *
 * Spinlocks                                                                                    *
 * ------------------------------------------------------------------------------------------- */

/// Creates a spinlock. Unlike the mutexes above, these are taken from interrupt context, so they
/// may not block.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_create_spinlock() -> uacpi_handle {
    todo!("Allocate a spinlock and return an opaque handle to it.")
}

/// Destroys a spinlock created by [`uacpi_kernel_create_spinlock`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_free_spinlock(handle: uacpi_handle) {
    let _ = handle;
    todo!("Destroy the spinlock behind the handle.")
}

/// Takes a spinlock, masking interrupts and returning the prior CPU flags. uACPI treats this as
/// infallible.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_lock_spinlock(handle: uacpi_handle) -> uacpi_cpu_flags {
    let _ = handle;
    todo!("Mask interrupts, take the spinlock, and return the saved CPU flags.")
}

/// Drops a spinlock and restores the flags [`uacpi_kernel_lock_spinlock`] returned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_unlock_spinlock(
    handle: uacpi_handle,
    flags: uacpi_cpu_flags,
) {
    let _ = (handle, flags);
    todo!("Release the spinlock and restore the saved CPU flags.")
}

/* ------------------------------------------------------------------------------------------- *
 * Interrupts                                                                                   *
 * ------------------------------------------------------------------------------------------- */

/// Installs `handler` on `irq` with `ctx` as its argument, returning a handle that
/// [`uacpi_kernel_uninstall_interrupt_handler`] can refer to it by.
///
/// This is how uACPI takes delivery of the SCI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_install_interrupt_handler(
    irq: uacpi_u32,
    handler: uacpi_interrupt_handler,
    ctx: uacpi_handle,
    out_irq_handle: *mut uacpi_handle,
) -> uacpi_status {
    let _ = (irq, handler, ctx, out_irq_handle);
    todo!("Route the IRQ to the supplied handler and hand back a handle identifying it.")
}

/// Removes a handler installed by [`uacpi_kernel_install_interrupt_handler`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_uninstall_interrupt_handler(
    handler: uacpi_interrupt_handler,
    irq_handle: uacpi_handle,
) -> uacpi_status {
    let _ = (handler, irq_handle);
    todo!("Tear down the interrupt routing this handle identifies.")
}

/* ------------------------------------------------------------------------------------------- *
 * Deferred work                                                                                *
 * ------------------------------------------------------------------------------------------- */

/// Queues `handler` to run later with `ctx`. This may be called from interrupt context, and
/// `UACPI_WORK_GPE_EXECUTION` in particular has to land on the bootstrap LP to sidestep firmware
/// that assumes as much.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_schedule_work(
    work_type: uacpi_work_type,
    handler: uacpi_work_handler,
    ctx: uacpi_handle,
) -> uacpi_status {
    let _ = (work_type, handler, ctx);
    todo!("Queue the work item, pinning GPE work to the bootstrap LP.")
}

/// Drains, in this order, every in-flight interrupt installed through
/// [`uacpi_kernel_install_interrupt_handler`] and then every item queued through
/// [`uacpi_kernel_schedule_work`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_wait_for_work_completion() -> uacpi_status {
    todo!("Wait out in-flight interrupts first, then the deferred work queue.")
}

/* ------------------------------------------------------------------------------------------- *
 * Firmware requests                                                                            *
 * ------------------------------------------------------------------------------------------- */

/// Handles a request AML made of the OS: a `Breakpoint` op or a `Fatal` op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uacpi_kernel_handle_firmware_request(
    request: *mut uacpi_firmware_request,
) -> uacpi_status {
    let _ = request;
    todo!("Act on the breakpoint or fatal error the firmware raised.")
}
