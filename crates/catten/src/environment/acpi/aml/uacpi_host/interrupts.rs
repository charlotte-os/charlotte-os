//! ACPI SCI/GPE routing and lifetime synchronization.

use core::sync::atomic::{AtomicUsize, Ordering};

use uacpi_wrapper::*;

static IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);

pub(super) fn wait_for_interrupts() {
    while IN_FLIGHT.load(Ordering::Acquire) != 0 {
        crate::cpu::scheduler::yield_lp();
    }
}

#[cfg(target_arch = "x86_64")]
mod platform {
    use super::super::c_alloc::{allocate_object, free_object};
    use super::*;
    use crate::cpu::isa::constants::interrupt_vectors::{DYN_VEC_START_OFFSET, DYN_VECS_PER_LP};
    use crate::cpu::isa::interface::interrupts::{DynIhMapIfce, ExternalInterruptControllerIfce};
    use crate::cpu::isa::interrupts::dynamic::DYN_IH_MAP;
    use crate::cpu::multiprocessor::spin::mutex::Mutex;
    use crate::device_management::drivers::platform_devices::wired_interrupt_controller::ioapic::{
        IOAPIC_LIST,
        IoapicId,
    };
    use crate::device_management::interrupt_routing::InterruptTarget;
    use crate::environment::acpi::sdt::madt::interface::resolve_acpi_interrupt;

    // The BSP is LP 0. The registry stores opaque addresses so an interrupt can acquire an
    // in-flight reference under its lock without allocating or retaining a lock during AML.
    static HANDLERS: Mutex<[Option<usize>; DYN_VECS_PER_LP as usize]> =
        Mutex::new([None; DYN_VECS_PER_LP as usize]);

    struct Registration {
        handler: unsafe extern "C" fn(uacpi_handle) -> uacpi_interrupt_ret,
        context: usize,
        gsi: u32,
        ioapic: IoapicId,
        pin: u8,
        vector: u8,
        active: AtomicUsize,
    }

    extern "C" fn dispatch(vector: u8) {
        let registration = {
            let handlers = HANDLERS.lock();
            let Some(address) = handlers[(vector - DYN_VEC_START_OFFSET) as usize] else {
                return;
            };
            let registration = unsafe { &*(address as *const Registration) };
            registration.active.fetch_add(1, Ordering::Acquire);
            IN_FLIGHT.fetch_add(1, Ordering::Acquire);
            registration
        };
        // Uninstall removes the registry entry before waiting for active to reach zero, so
        // both the registration and the uACPI context remain valid until this call returns.
        unsafe { (registration.handler)(registration.context as uacpi_handle) };
        IN_FLIGHT.fetch_sub(1, Ordering::Release);
        registration.active.fetch_sub(1, Ordering::Release);
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn uacpi_kernel_install_interrupt_handler(
        irq: uacpi_u32,
        handler: uacpi_interrupt_handler,
        context: uacpi_handle,
        out_handle: *mut uacpi_handle,
    ) -> uacpi_status {
        let Some(handler) = handler else {
            return UACPI_STATUS_INVALID_ARGUMENT;
        };
        if out_handle.is_null() {
            return UACPI_STATUS_INVALID_ARGUMENT;
        }
        unsafe { out_handle.write(core::ptr::null_mut()) };
        let Some((gsi, active_low, level)) = resolve_acpi_interrupt(irq) else {
            return UACPI_STATUS_INVALID_ARGUMENT;
        };
        let ioapics = IOAPIC_LIST.read();
        let Some((id, pin)) = ioapics
            .iter()
            .find_map(|(id, controller)| controller.lock().pin_for_gsi(gsi).map(|pin| (*id, pin)))
        else {
            return UACPI_STATUS_NOT_FOUND;
        };
        let registration = allocate_object(Registration {
            handler,
            context: context as usize,
            gsi,
            ioapic: id,
            pin,
            vector: 0,
            active: AtomicUsize::new(0),
        });
        if registration.is_null() {
            return UACPI_STATUS_OUT_OF_MEMORY;
        }
        let mut handlers = HANDLERS.lock();
        if handlers
            .iter()
            .flatten()
            .any(|&address| unsafe { (*(address as *const Registration)).gsi == gsi })
        {
            unsafe { free_object(registration) };
            return UACPI_STATUS_ALREADY_EXISTS;
        }
        let mut vectors = DYN_IH_MAP.write();
        let Some(InterruptTarget::Processor {
            discriminator: vector,
            ..
        }) = vectors.find_available_target_on_lp(0)
        else {
            unsafe { free_object(registration) };
            return UACPI_STATUS_OUT_OF_MEMORY;
        };
        let mut controller = ioapics[&id].lock();
        if controller.setup_ext_int(0, vector, pin, active_low, level, true).is_err() {
            unsafe { free_object(registration) };
            return UACPI_STATUS_UNIMPLEMENTED;
        }
        if vectors.set_dyn_ih(0, vector, dispatch).is_err() {
            unsafe { free_object(registration) };
            return UACPI_STATUS_INTERNAL_ERROR;
        }
        unsafe { (*registration).vector = vector };
        handlers[(vector - DYN_VEC_START_OFFSET) as usize] = Some(registration as usize);
        // The callback and lifetime record are visible before hardware can deliver the SCI.
        controller.set_ext_int_mask_state(pin, false).unwrap_or_else(|_| unreachable!());
        unsafe { out_handle.write(registration.cast()) };
        UACPI_STATUS_OK
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn uacpi_kernel_uninstall_interrupt_handler(
        handler: uacpi_interrupt_handler,
        handle: uacpi_handle,
    ) -> uacpi_status {
        let Some(handler) = handler else {
            return UACPI_STATUS_INVALID_ARGUMENT;
        };
        let registration = {
            let mut handlers = HANDLERS.lock();
            let Some(index) = handlers.iter().position(|entry| *entry == Some(handle as usize))
            else {
                return UACPI_STATUS_NOT_FOUND;
            };
            let registration = unsafe { &*(handle as *const Registration) };
            if !core::ptr::fn_addr_eq(registration.handler, handler) {
                return UACPI_STATUS_INVALID_ARGUMENT;
            }
            IOAPIC_LIST.read()[&registration.ioapic]
                .lock()
                .set_ext_int_mask_state(registration.pin, true)
                .unwrap_or_else(|_| unreachable!());
            handlers[index] = None;
            DYN_IH_MAP
                .write()
                .clear_dyn_ih(0, registration.vector)
                .unwrap_or_else(|_| unreachable!());
            registration
        };
        while registration.active.load(Ordering::Acquire) != 0 {
            crate::cpu::scheduler::yield_lp();
        }
        unsafe { free_object(handle as *mut Registration) };
        UACPI_STATUS_OK
    }
}

#[cfg(not(target_arch = "x86_64"))]
mod platform {
    use super::*;

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn uacpi_kernel_install_interrupt_handler(
        _irq: uacpi_u32,
        _handler: uacpi_interrupt_handler,
        _context: uacpi_handle,
        out_handle: *mut uacpi_handle,
    ) -> uacpi_status {
        if !out_handle.is_null() {
            unsafe { out_handle.write(core::ptr::null_mut()) };
        }
        UACPI_STATUS_UNIMPLEMENTED
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn uacpi_kernel_uninstall_interrupt_handler(
        _handler: uacpi_interrupt_handler,
        _handle: uacpi_handle,
    ) -> uacpi_status {
        UACPI_STATUS_UNIMPLEMENTED
    }
}
