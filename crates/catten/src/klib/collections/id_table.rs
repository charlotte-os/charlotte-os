use alloc::vec::Vec;
use core::fmt::Debug;

use crate::logln;

#[derive(Debug)]
pub enum Error {
    IdNotActive,
}

#[derive(Debug)]
pub struct IdTable<T> {
    list: Vec<Option<T>>,
    available_ids: Vec<usize>,
}

impl<'a, T> IdTable<T> {
    pub fn new() -> Self {
        IdTable {
            list: Vec::new(),
            available_ids: Vec::new(),
        }
    }

    pub fn add_element(&mut self, element: T) -> usize {
        logln!("Adding element to ID Table.");
        if let Some(id) = self.available_ids.pop() {
            logln!("ID Table: Available ID found: {id}.");
            self.list[id] = Some(element);
            logln!("ID Table: Added element to list.");
            id
        } else {
            logln!("ID Table: No available IDs. Extending list to push element.");
            let id = self.list.len();
            self.list.push(Some(element));
            logln!("ID Table: Added element to list.");
            id
        }
    }

    pub fn get(&self, element_id: usize) -> Result<&T, Error> {
        self.list.get(element_id).ok_or(Error::IdNotActive)?.as_ref().ok_or(Error::IdNotActive)
    }

    pub fn get_mut(&mut self, element_id: usize) -> Result<&mut T, Error> {
        self.list.get_mut(element_id).ok_or(Error::IdNotActive)?.as_mut().ok_or(Error::IdNotActive)
    }

    pub fn remove_element(&mut self, element_id: usize) -> Result<(), Error> {
        let element = self.list.get_mut(element_id).ok_or(Error::IdNotActive)?;
        if element.is_none() {
            return Err(Error::IdNotActive);
        }
        *element = None;
        self.available_ids.push(element_id);
        Ok(())
    }

    pub fn iter(&'a self) -> core::slice::Iter<'a, Option<T>> {
        self.list.iter()
    }

    pub fn iter_mut(&'a mut self) -> core::slice::IterMut<'a, Option<T>> {
        self.list.iter_mut()
    }
}

unsafe impl<T> Send for IdTable<T> where T: Send {}
unsafe impl<T> Sync for IdTable<T> where T: Sync {}
