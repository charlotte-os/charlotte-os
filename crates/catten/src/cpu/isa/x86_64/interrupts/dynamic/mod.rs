mod stubs;

use alloc::{
    boxed::Box,
    vec,
};
use core::{
    mem::{
        transmute,
        type_info::Int,
    },
    num::NonZeroUsize,
    ops::{
        Index,
        IndexMut,
    },
    sync::atomic::{
        Atomic,
        AtomicUsize,
        Ordering,
    },
};

use spin::LazyLock;

use crate::{
    cpu::{
        interrupt_routing::InterruptHandler,
        isa::{
            constants::interrupt_vectors::{
                DYN_VEC_START_OFFSET,
                DYN_VECS_PER_LP,
                FIXED_INTERRUPT_VECTORS,
            },
            interface::interrupts::DynIhMapIfce,
            interrupts::Error,
            lp::{
                IntSrcDscr,
                LpId,
                ops::get_lp_id,
            },
        },
        multiprocessor::get_lp_count,
    },
    klib::collections::boxed_slice::make_boxed_slice,
};

/// The instance of the dynamic interrupt handler matrix
#[unsafe(no_mangle)]
pub static DYN_IH_MAP: LazyLock<DynIhMap> = LazyLock::new(DynIhMap::new);

#[derive(Debug, Clone, Copy)]
struct LpDynIhTable {
    vectors_used: u8,
    table: [Option<InterruptHandler>; DYN_VECS_PER_LP as usize],
}

impl LpDynIhTable {
    fn vectors_in_use(&self) -> u8 {
        self.vectors_used
    }

    fn get(&self, index: usize) -> &Option<InterruptHandler> {
        &self.table[index]
    }

    fn get_copied(&self, index: usize) -> Option<InterruptHandler> {
        self.table[index].clone()
    }

    fn get_mut(&mut self, index: usize) -> &mut Option<InterruptHandler> {
        &mut self.table[index]
    }
}

impl Index<usize> for LpDynIhTable {
    type Output = Option<InterruptHandler>;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl IndexMut<usize> for LpDynIhTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
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
    pub fn get_handler(&self, vec: IntSrcDscr) -> Result<InterruptHandler, Error> {
        if core::hint::likely(Self::in_dyn_vec_range(vec)) {
            let curr_lp = get_lp_id() as usize;
            let vec_index = (vec - DYN_VEC_START_OFFSET) as usize;
            self.map_table[curr_lp].get_copied(vec_index).ok_or(Error::IntVecUnassigned(vec))
        } else {
            Err(Error::ArgIsFixedIntVec(vec))
        }
    }
}

impl DynIhMapIfce for DynIhMap {
    extern "C" fn get_dyn_ih(&self, vector: IntSrcDscr) -> Option<InterruptHandler> {
        match self.get_handler(vector) {
            Ok(handler) => Some(handler),
            Err(_) => None,
        }
    }
}
