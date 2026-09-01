mod irte;

use core::ptr::NonNull;

use super::Error;
use crate::cpu::isa::interface::memory::address::VirtualAddressIfce;
use crate::cpu::multiprocessor::spin::mutex::Mutex;
use crate::cpu::multiprocessor::spin::rwlock::RwLock;
use crate::device_management::drivers::busses::pci_express::device_class::device_class::IOAPIC;
use crate::klib::bitwise::mask_shift_read;
use crate::memory::VirtualAddress;

//pub static IOAPIC_LIST: RwLock<HashMap<IoapicId, Mutex<IoapicDescriptor>>>

pub type IoapicId = u8;

pub struct IoapicDescriptor {
    base: VirtualAddress,
    version: u8,
    num_redirection_entries: u8,
}

impl IoapicDescriptor {
    /// Create a new IOAPIC descriptor from the given base address.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the base address is valid and points to a properly mapped IOAPIC
    /// MMIO region.
    unsafe fn new(base: VirtualAddress) -> Self {
        // save the MMIO addresses
        let io_reg_sel_addr = base.clone().into_mut::<u32>();
        let io_reg_win_addr = (base + 0x10usize).into_mut::<u32>();
        // read the version register
        let version_reg = unsafe {
            io_reg_sel_addr.write_volatile(0x01);
            io_reg_win_addr.read_volatile()
        };
        // extract the version and number of redirection entries from the version register
        // value and construct the descriptor
        const IOAPIC_VERSION_MASK: u32 = 0xff;
        const IOAPIC_REDIRECTION_ENTRY_MAX_MASK: u32 = 0xff;
        const IOAPIC_REDIRECTION_ENTRY_MAX_SHIFT: u8 = 16;
        Self {
            base,
            version: (version_reg & IOAPIC_VERSION_MASK) as u8,
            num_redirection_entries: (mask_shift_read(
                version_reg,
                IOAPIC_REDIRECTION_ENTRY_MAX_MASK,
                IOAPIC_REDIRECTION_ENTRY_MAX_SHIFT,
            ) + 1) as u8,
        }
    }

    /// Get the MMIO addresses for the IOAPIC registers.
    fn get_mmio(&self) -> (NonNull<u32>, NonNull<u32>) {
        unsafe {
            let io_reg_sel_addr = NonNull::new_unchecked(self.base.clone().into_mut::<u32>());
            let io_reg_win_addr =
                NonNull::new_unchecked((self.base.clone() + 0x10usize).into_mut::<u32>());
            (io_reg_sel_addr, io_reg_win_addr)
        }
    }

    /// Write a 32-bit value to the specified IOAPIC register.
    fn write_reg32(&mut self, offset: u32, value: u32) {
        let (io_reg_sel_addr, io_reg_win_addr) = self.get_mmio();
        unsafe {
            io_reg_sel_addr.write_volatile(offset);
            io_reg_win_addr.write_volatile(value);
        }
    }

    /// Read a 32-bit value from the specified IOAPIC register.
    fn read_reg32(&mut self, offset: u32) -> u32 {
        let (io_reg_sel_addr, io_reg_win_addr) = self.get_mmio();
        unsafe {
            io_reg_sel_addr.write_volatile(offset);
            io_reg_win_addr.read_volatile()
        }
    }

    fn signal_eoi(&mut self, vector: u8) {
        const IOXAPIC_THRESHOLD_VERSION: u8 = 0x20;
        if self.version >= IOXAPIC_THRESHOLD_VERSION {
            let ioapic_eoi_reg_ptr = (self.base + 0x40usize).into_mut::<u32>();
            unsafe {
                ioapic_eoi_reg_ptr.write_volatile(vector as u32);
            }
        } else {
            todo!("Use the trick Linux uses for signaling EOI on older IOAPIC versions")
        }
    }
}
