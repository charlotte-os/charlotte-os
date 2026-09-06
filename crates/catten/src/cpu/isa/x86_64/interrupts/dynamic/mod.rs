mod stubs;

use alloc::boxed::Box;
use core::ops::{Index, IndexMut};

use spin::LazyLock;

use crate::cpu::isa::constants::interrupt_vectors::{DYN_VEC_START_OFFSET, DYN_VECS_PER_LP};
use crate::cpu::isa::interface::interrupts::DynIhMapIfce;
use crate::cpu::isa::interrupts::Error;
use crate::cpu::isa::lp::ops::get_lp_id;
use crate::cpu::isa::lp::{IntSrcDscr, LpId};
use crate::cpu::multiprocessor::get_lp_count;
use crate::cpu::multiprocessor::spin::rwlock::RwLock;
use crate::device_management::interrupt_routing::{InterruptHandler, InterruptTarget};
use crate::klib::collections::boxed_slice::make_boxed_slice;

/// The instance of the dynamic interrupt handler matrix
#[unsafe(no_mangle)]
pub static DYN_IH_MAP: LazyLock<RwLock<DynIhMap>> = LazyLock::new(|| RwLock::new(DynIhMap::new()));

#[derive(Debug, Clone, Copy)]
struct LpDynIhTable {
    vectors_used: u8,
    table: [Option<InterruptHandler>; DYN_VECS_PER_LP as usize],
}

impl LpDynIhTable {
    fn vectors_in_use(&self) -> u8 {
        self.vectors_used
    }

    fn get(&self, index: IntSrcDscr) -> &Option<InterruptHandler> {
        &self.table[index as usize]
    }

    fn get_copied(&self, index: IntSrcDscr) -> Option<InterruptHandler> {
        self.table[index as usize].clone()
    }

    fn get_mut(&mut self, index: IntSrcDscr) -> &mut Option<InterruptHandler> {
        &mut self.table[index as usize]
    }
}

impl Index<IntSrcDscr> for LpDynIhTable {
    type Output = Option<InterruptHandler>;

    fn index(&self, index: IntSrcDscr) -> &Self::Output {
        self.get(index)
    }
}

impl IndexMut<IntSrcDscr> for LpDynIhTable {
    fn index_mut(&mut self, index: IntSrcDscr) -> &mut Self::Output {
        self.get_mut(index)
    }
}

/// The dynamic interrupt handler map is used by the by the stub ISRs in IDT mode or the
/// interrupt entry points in FRED mode to find and call their respective dynamically registered
/// interrupt handlers.
#[derive(Debug)]
pub struct DynIhMap {
    map_table: Box<[LpDynIhTable]>,
}

impl DynIhMap {
    /// Creates a new dynamic interrupt handler map with all entries initialized to `None`.
    fn new() -> Self {
        Self {
            map_table: make_boxed_slice(get_lp_count() as usize, || LpDynIhTable {
                vectors_used: 0,
                table: [None; DYN_VECS_PER_LP as usize],
            }),
        }
    }

    pub const fn in_dyn_vec_range(vec: IntSrcDscr) -> bool {
        vec >= DYN_VEC_START_OFFSET && vec < DYN_VEC_START_OFFSET + DYN_VECS_PER_LP
    }

    /// Attempts to retrieve the interrupt handler for the given dynamic vector and returns a rich
    /// error type if needed. Not directly callable from assembly code; use `get_dyn_ih` for
    /// that purpose.
    pub fn get_handler(&self, lp: LpId, vector: IntSrcDscr) -> Result<InterruptHandler, Error> {
        if core::hint::likely(Self::in_dyn_vec_range(vector)) {
            self.map_table[lp as usize].get_copied(vector).ok_or(Error::IntVecUnassigned(vector))
        } else {
            Err(Error::ArgIsFixedIntVec(vector))
        }
    }

    fn find_least_loaded_lp_idx(&self) -> usize {
        let map_table = &self.map_table;
        map_table
            .iter()
            .enumerate()
            .min_by_key(|&(_, lp_table)| lp_table.vectors_used)
            .map(|(i, _)| i)
            .unwrap()
    }
}

impl DynIhMapIfce for DynIhMap {
    fn set_dyn_ih(
        &mut self,
        lp: LpId,
        vector: IntSrcDscr,
        handler: InterruptHandler,
    ) -> Result<(), Error> {
        if !Self::in_dyn_vec_range(vector) {
            return Err(Error::ArgIsFixedIntVec(vector));
        }

        let lp_index = lp as usize;
        let lp_table = &mut self.map_table[lp_index];

        if lp_table.get_mut(vector).is_none() {
            *lp_table.get_mut(vector) = Some(handler);
            lp_table.vectors_used += 1;
        }
        Ok(())
    }

    #[unsafe(no_mangle)]
    extern "C" fn get_local_dyn_ih(&self, vector: IntSrcDscr) -> Option<InterruptHandler> {
        match self.get_handler(get_lp_id(), vector) {
            Ok(handler) => Some(handler),
            Err(_) => None,
        }
    }

    fn clear_dyn_ih(&mut self, lp: LpId, vector: IntSrcDscr) -> Result<(), Error> {
        if !Self::in_dyn_vec_range(vector) {
            return Err(Error::ArgIsFixedIntVec(vector));
        }

        let lp_index = lp as usize;
        let lp_table = &mut self.map_table[lp_index];

        if lp_table.get_mut(vector).is_some() {
            *lp_table.get_mut(vector) = None;
            lp_table.vectors_used -= 1;
        }
        Ok(())
    }

    fn find_available_target(&self) -> Option<InterruptTarget> {
        let lp_idx = self.find_least_loaded_lp_idx();
        let lp_table = &self.map_table[lp_idx];
        for i in DYN_VEC_START_OFFSET..(DYN_VEC_START_OFFSET + DYN_VECS_PER_LP) {
            if lp_table.get(i).is_none() {
                return Some(InterruptTarget::Processor {
                    lp_id: lp_idx as LpId,
                    discriminator: i,
                });
            }
        }
        None
    }
}
