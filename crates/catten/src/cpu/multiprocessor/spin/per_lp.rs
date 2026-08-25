use alloc::boxed::Box;

use crate::cpu::isa::lp::LpId;
use crate::cpu::isa::lp::ops::get_lp_id;
use crate::cpu::multiprocessor::get_lp_count;
use crate::cpu::multiprocessor::spin::rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::klib::collections::boxed_slice::make_boxed_slice;

#[derive(Debug)]
pub enum Error {
    TargetBusy,
    InvalidIndex,
}

#[derive(Debug)]
pub struct PerLp<T> {
    data: Box<[RwLock<T>]>,
}

/// A structure that provides per-Logical Processor (LP) sharded memory for a value of type `T`.
impl<'a, T> PerLp<T> {
    pub fn new<F: Fn() -> T>(initializer: F) -> Self {
        PerLp {
            data: make_boxed_slice(get_lp_count() as usize, || RwLock::new(initializer())),
        }
    }

    pub fn get(&'a self) -> RwLockReadGuard<'a, T> {
        self.data[get_lp_id() as usize].read()
    }

    pub fn get_mut(&'a self) -> RwLockWriteGuard<'a, T> {
        self.data[get_lp_id() as usize].write()
    }

    pub fn try_get(&'a self) -> Result<RwLockReadGuard<'a, T>, Error> {
        match self.data[get_lp_id() as usize].try_read() {
            Some(guard) => Ok(guard),
            None => Err(Error::TargetBusy),
        }
    }

    pub fn try_get_mut(&'a self) -> Result<RwLockWriteGuard<'a, T>, Error> {
        match self.data[get_lp_id() as usize].try_write() {
            Some(guard) => Ok(guard),
            None => Err(Error::TargetBusy),
        }
    }

    pub unsafe fn get_nonlocal(&'a self, lp_id: LpId) -> RwLockReadGuard<'a, T> {
        let lp_index = lp_id as usize;
        if lp_index >= self.data.len() {
            panic!("Invalid LP index");
        }
        self.data[lp_index].read()
    }

    pub unsafe fn get_nonlocal_mut(&'a self, lp_id: LpId) -> RwLockWriteGuard<'a, T> {
        let lp_index = lp_id as usize;
        if lp_index >= self.data.len() {
            panic!("Invalid LP index");
        }
        self.data[lp_index].write()
    }

    pub unsafe fn try_get_nonlocal(&'a self, lp_id: LpId) -> Result<RwLockReadGuard<'a, T>, Error> {
        let lp_index = lp_id as usize;
        if lp_index >= self.data.len() {
            return Err(Error::InvalidIndex);
        }
        match self.data[lp_index].try_read() {
            Some(guard) => Ok(guard),
            None => Err(Error::TargetBusy),
        }
    }

    pub unsafe fn try_get_nonlocal_mut(
        &'a self,
        lp_id: LpId,
    ) -> Result<RwLockWriteGuard<'a, T>, Error> {
        let lp_index = lp_id as usize;
        if lp_index >= self.data.len() {
            return Err(Error::InvalidIndex);
        }
        match self.data[lp_index].try_write() {
            Some(guard) => Ok(guard),
            None => Err(Error::TargetBusy),
        }
    }
}

impl<T: Default> Default for PerLp<T> {
    fn default() -> Self {
        Self::new(T::default)
    }
}
