use core::ptr::NonNull;

use super::Error;
use crate::cpu::multiprocessor::spin::{mutex::Mutex, rwlock::RwLock};

//pub static IOAPIC_LIST: RwLock<Vec<Mutex<IoapicDescriptor>>>

pub struct IoapicDescriptor {
    id: u8,
    io_reg_sel_addr: NonNull<u32>,
    io_reg_win_addr: NonNull<u32>
}

impl IoapicDescriptor {
    unsafe fn write_reg32(&mut self, offset: u32, value: u32) {
        unsafe {
            self.io_reg_sel_addr.write(offset);
            self.io_reg_win_addr.write(value);
        }
    }
    unsafe fn read_reg32(&mut self, offset: u32) -> u32 {
        unsafe {
            self.io_reg_sel_addr.write(offset);
            self.io_reg_win_addr.read()
        }
    }

}
