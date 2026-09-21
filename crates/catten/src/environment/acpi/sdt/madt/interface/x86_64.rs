use alloc::vec::Vec;
use core::ops::Add;

use hashbrown::HashMap;

use crate::cpu::isa::interface::memory::address::Address;
use crate::cpu::multiprocessor::spin::mutex::Mutex;
#[cfg(target_arch = "x86_64")]
use crate::device_management::drivers::platform_devices::wired_interrupt_controller::ioapic::{
    IoapicDescriptor,
    IoapicId,
};
use crate::device_management::interrupt_routing::legacy_irqs::LEGACY_IRQ_COUNT;
use crate::environment::acpi::sdt::GlobalSystemInterrupt;
use crate::environment::acpi::sdt::madt::entries::interrupt_source_override::InterruptSourceOverrideEntry;
use crate::environment::acpi::sdt::madt::entries::ioapic::IoApicEntry;
use crate::environment::acpi::sdt::madt::entries::{MADT_INDEX, MadtEntryType, interrupt_flags};
use crate::logln;
use crate::memory::allocators::memory::PageSize;
use crate::memory::linear::address_map::RegionType::KernelMmio;
use crate::memory::linear::address_map::{LA_MAP, LinearMemoryRegion};
use crate::memory::linear::{MemoryMapping, PageType};
use crate::memory::{AddressSpaceInterface, KERNEL_AS, PhysicalAddress, VirtualAddress};

#[derive(Debug)]
pub struct IrqGsiMapping {
    pub irq: u8,
    pub gsi: GlobalSystemInterrupt,
    pub polarity: bool,
    pub latched: bool,
}

/// Create a mapping table from legacy IRQ numbers to ACPI Global System Interrupts (GSIs)
pub fn create_irq_gsi_mapping_table() -> [IrqGsiMapping; LEGACY_IRQ_COUNT] {
    // Set legacy defaults
    let mut irq_override_table = core::array::from_fn(|irq| IrqGsiMapping {
        irq: irq as u8,
        gsi: irq as GlobalSystemInterrupt,
        polarity: true,
        latched: false,
    });
    // Override with entries from the MADT
    let irq_override_entries =
        (*MADT_INDEX).get_entries_with_type(MadtEntryType::InterruptSourceOverride);
    for entry in irq_override_entries.iter() {
        let override_entry = *entry as *const InterruptSourceOverrideEntry;
        if (unsafe { (*entry).read_unaligned() }.entry_length as usize)
            < core::mem::size_of::<InterruptSourceOverrideEntry>()
        {
            continue;
        }
        let irq = unsafe { override_entry.read_unaligned() }.irq_source as usize;
        if irq < LEGACY_IRQ_COUNT {
            irq_override_table[irq].gsi =
                unsafe { override_entry.read_unaligned() }.global_system_interrupt;
            let flags = unsafe { override_entry.read_unaligned() }.flags;
            irq_override_table[irq].polarity =
                flags.polarity() != interrupt_flags::InterruptPolarity::ActiveLow;
            irq_override_table[irq].latched =
                flags.trigger_mode() == interrupt_flags::InterruptTriggerMode::Level;
        }
    }
    // Return the final IRQ to GSI mapping table
    irq_override_table
}

/// Resolve an ACPI event interrupt. SCI/GPE lines default to active-low, level-triggered;
/// only explicit MADT flags replace those defaults. Legacy ISA defaults are inappropriate here.
pub fn resolve_acpi_interrupt(irq: u32) -> Option<(u32, bool, bool)> {
    let mut route = (irq, true, true);
    if irq >= LEGACY_IRQ_COUNT as u32 {
        return Some(route);
    }
    for entry in MADT_INDEX.get_entries_with_type(MadtEntryType::InterruptSourceOverride) {
        if (unsafe { (*entry).read_unaligned() }.entry_length as usize)
            < core::mem::size_of::<InterruptSourceOverrideEntry>()
        {
            return None;
        }
        let entry = unsafe { (*entry as *const InterruptSourceOverrideEntry).read_unaligned() };
        if entry.bus != 0 || entry.irq_source as u32 != irq {
            continue;
        }
        let flags = entry.flags;
        route.0 = entry.global_system_interrupt;
        route.1 = match flags.polarity() {
            interrupt_flags::InterruptPolarity::BusSpec => true,
            interrupt_flags::InterruptPolarity::ActiveHigh => false,
            interrupt_flags::InterruptPolarity::ActiveLow => true,
            interrupt_flags::InterruptPolarity::Reserved => return None,
        };
        route.2 = match flags.trigger_mode() {
            interrupt_flags::InterruptTriggerMode::BusSpec => true,
            interrupt_flags::InterruptTriggerMode::Edge => false,
            interrupt_flags::InterruptTriggerMode::Level => true,
            interrupt_flags::InterruptTriggerMode::Reserved => return None,
        };
        break;
    }
    Some(route)
}

pub unsafe fn enumerate_ioapics() -> HashMap<IoapicId, Mutex<IoapicDescriptor>> {
    logln!("[ACPI] Enumerating IOAPICs via the ACPI MADT.");

    let ioapic_madt_entries = unsafe {
        core::mem::transmute::<_, &Vec<*const IoApicEntry>>(
            MADT_INDEX.get_entries_with_type(MadtEntryType::IoApic),
        )
    };
    logln!("[ACPI] Found {} IOAPIC entries in the MADT.", (ioapic_madt_entries.len()));
    let mut ioapic_map = HashMap::default();
    let mut mapped_io_pages = HashMap::<PhysicalAddress, VirtualAddress>::default();
    for e in ioapic_madt_entries.iter() {
        let vaddr = if mapped_io_pages.contains_key(
            &PhysicalAddress::from(unsafe { e.read_unaligned() }.ioapic_address as u64)
                .prev_aligned_to(PageSize::Standard.num_bytes()),
        ) {
            mapped_io_pages
                .get(
                    &PhysicalAddress::from(unsafe { e.read_unaligned() }.ioapic_address as u64)
                        .prev_aligned_to(PageSize::Standard.num_bytes()),
                )
                .unwrap()
                .add(unsafe { e.read_unaligned() }.ioapic_address as usize & 0xfff)
        } else {
            // Map the IOAPIC MMIO region and store the mapped page
            let mut kas = KERNEL_AS.lock();
            let vaddr = kas
                .find_free_region(
                    1,
                    <LinearMemoryRegion as Into<(VirtualAddress, VirtualAddress)>>::into(
                        *LA_MAP.get_region(KernelMmio),
                    ),
                )
                .expect(
                    "Failed to find available address space region while attempting to map IOAPIC \
                     MMIO",
                );
            let mapping = MemoryMapping {
                vaddr,
                paddr: PhysicalAddress::from(unsafe { e.read_unaligned() }.ioapic_address as u64)
                    .prev_aligned_to(PageSize::Standard.num_bytes()),
                page_type: PageType::Mmio,
            };
            kas.map_page(mapping)
                .expect("Failed to map IOAPIC MMIO region into the kernel address space.");
            mapped_io_pages.insert(
                PhysicalAddress::from(unsafe { e.read_unaligned() }.ioapic_address as u64)
                    .prev_aligned_to(PageSize::Standard.num_bytes()),
                vaddr,
            );
            vaddr + (unsafe { e.read_unaligned() }.ioapic_address as usize & 0xfff)
        };
        let ioapic_desc = unsafe {
            IoapicDescriptor::new(vaddr, unsafe { e.read_unaligned() }.global_system_interrupt_base)
        };
        logln!(
            "[ACPI] Enumerated IOAPIC with ID = {:?}: {:?}",
            (unsafe { e.read_unaligned() }.ioapic_id),
            (ioapic_desc)
        );
        ioapic_map.insert(unsafe { e.read_unaligned() }.ioapic_id, Mutex::new(ioapic_desc));
    }
    ioapic_map
}
