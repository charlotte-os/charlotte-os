use hashbrown::HashMap;
use spin::LazyLock;

use crate::cpu::isa::interface::memory::address::Address;
use crate::cpu::multiprocessor::spin::rwlock::RwLock;
use crate::memory::PhysicalAddress;
use crate::memory::linear::PageType;
use crate::memory::physical::PAGE_FRAME_SIZE;

pub static SHARED_MEMORY_MANAGER: LazyLock<SharedMemoryManager> =
    LazyLock::new(SharedMemoryManager::default);

#[derive(Debug, Clone, Copy)]
pub enum Error {
    InvalidFrameNum,
    FrameAlreadyShared,
    FrameNotShared,
}

pub struct SharedPhysFrame {
    pub page_type: PageType,
    pub handle_count: u64,
    pub mapping_count: u64,
}

#[derive(Default)]
pub struct SharedMemoryManager {
    frames: RwLock<HashMap<PhysicalAddress, RwLock<SharedPhysFrame>>>,
}

impl SharedMemoryManager {
    pub fn mark_shared(
        &self,
        frame_num: PhysicalAddress,
        frame: SharedPhysFrame,
    ) -> Result<(), Error> {
        if !frame_num.is_aligned_to(PAGE_FRAME_SIZE) {
            Err(Error::InvalidFrameNum)
        } else if self.frames.write().try_insert(frame_num, RwLock::new(frame)).is_err() {
            Err(Error::FrameAlreadyShared)
        } else {
            Ok(())
        }
    }

    pub fn increment_handle_count(&self, frame_num: PhysicalAddress) -> Result<(), Error> {
        if let Some(frame) = self.frames.read().get(&frame_num) {
            frame.write().handle_count += 1;
            Ok(())
        } else {
            Err(Error::FrameNotShared)
        }
    }

    pub fn decrement_handle_count(&self, frame_num: PhysicalAddress) -> Result<(), Error> {
        if let Some(frame) = self.frames.read().get(&frame_num) {
            let mut frame = frame.write();
            if frame.handle_count > 0 {
                frame.handle_count -= 1;
                if frame.handle_count == 0 && frame.mapping_count == 0 {
                    self.frames.write().remove(&frame_num);
                }
                Ok(())
            } else {
                Err(Error::FrameNotShared)
            }
        } else {
            Err(Error::FrameNotShared)
        }
    }

    pub fn increment_mapping_count(&self, frame_num: PhysicalAddress) -> Result<(), Error> {
        if let Some(frame) = self.frames.read().get(&frame_num) {
            frame.write().mapping_count += 1;
            Ok(())
        } else {
            Err(Error::FrameNotShared)
        }
    }

    pub fn decrement_mapping_count(&self, frame_num: PhysicalAddress) -> Result<(), Error> {
        if let Some(frame) = self.frames.read().get(&frame_num) {
            let mut frame = frame.write();
            if frame.mapping_count > 0 {
                frame.mapping_count -= 1;
                if frame.handle_count == 0 && frame.mapping_count == 0 {
                    self.frames.write().remove(&frame_num);
                }
                Ok(())
            } else {
                Err(Error::FrameNotShared)
            }
        } else {
            Err(Error::FrameNotShared)
        }
    }

    pub fn is_shared(&self, frame_num: PhysicalAddress) -> Result<bool, Error> {
        if frame_num.is_aligned_to(PAGE_FRAME_SIZE) {
            Ok(self.frames.read().contains_key(&frame_num))
        } else {
            Err(Error::InvalidFrameNum)
        }
    }
}

unsafe impl Sync for SharedMemoryManager {}
