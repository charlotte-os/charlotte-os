mod stubs;

use spin::LazyLock;

use crate::cpu::{
    interrupt_routing::InterruptHandler,
    isa::{
        constants::interrupt_vectors::{
            DYN_VEC_START_OFFSET,
            DYN_VECS_PER_LP,
            FIXED_INTERRUPT_VECTORS,
        },
        interface::interrupts::DynInterruptDispatcherIfce,
        interrupts::Error,
        lp::{
            InterruptVectorNum,
            LpId,
        },
    },
    multiprocessor::spin::per_lp::PerLp,
};

/// The instance of the dynamic interrupt handler matrix
#[unsafe(no_mangle)]
pub static DYN_IH_MATRIX: LazyLock<DynInterruptDispatcher> =
    LazyLock::new(DynInterruptDispatcher::default);

/// A wrapper type for dynamic interrupt vector numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DynamicInterruptVectorNum(u8);
impl TryFrom<InterruptVectorNum> for DynamicInterruptVectorNum {
    type Error = Error;

    fn try_from(value: InterruptVectorNum) -> Result<Self, Self::Error> {
        if FIXED_INTERRUPT_VECTORS.contains(&value) {
            Err(Error::InvalidDynamicVectorNumber(value))
        } else {
            Ok(DynamicInterruptVectorNum(value))
        }
    }
}

impl Into<InterruptVectorNum> for DynamicInterruptVectorNum {
    fn into(self) -> InterruptVectorNum {
        InterruptVectorNum::from(self.0 - DYN_VEC_START_OFFSET)
    }
}

impl Into<usize> for DynamicInterruptVectorNum {
    fn into(self) -> usize {
        <DynamicInterruptVectorNum as Into<InterruptVectorNum>>::into(self) as usize
    }
}

/// The dynamic interrupt dispatcher that manages the assignment of
/// dynamic interrupt vectors to interrupt handlers for each logical processor.
#[derive(Debug)]
pub struct DynInterruptDispatcher {
    matrix: PerLp<[Option<InterruptHandler>; DYN_VECS_PER_LP as usize]>,
    dynamic_vectors_used: PerLp<u64>,
}

impl Default for DynInterruptDispatcher {
    fn default() -> Self {
        DynInterruptDispatcher {
            matrix: PerLp::new(|| [None; DYN_VECS_PER_LP as usize]),
            dynamic_vectors_used: PerLp::new(|| 0),
        }
    }
}

impl DynInterruptDispatcherIfce for DynInterruptDispatcher {
    #[unsafe(no_mangle)]
    extern "C" fn set_dyn_ih(
        &self,
        lp: LpId,
        vector: InterruptVectorNum,
        handler: InterruptHandler,
    ) -> core::ffi::c_int {
        if let Ok(dyn_vec_num) = DynamicInterruptVectorNum::try_from(vector) {
            let mut table = unsafe { self.matrix.get_nonlocal_mut(lp) };
            let index = <DynamicInterruptVectorNum as Into<usize>>::into(dyn_vec_num);
            table[index] = Some(handler);
            0
        } else {
            -1
        }
    }

    #[unsafe(no_mangle)]
    extern "C" fn get_dyn_ih(&self, vector: InterruptVectorNum) -> *const InterruptHandler {
        if let Ok(table) = self.matrix.try_get() {
            if let Ok(dyn_vec_num) = DynamicInterruptVectorNum::try_from(vector) {
                let index: usize = <DynamicInterruptVectorNum as Into<usize>>::into(dyn_vec_num);
                if let Some(ih) = table[index] {
                    ih as *const InterruptHandler;
                }
            }
        }
        core::ptr::null()
    }

    fn is_vector_available(&self, lp: LpId, vector: InterruptVectorNum) -> bool {
        let table = unsafe { self.matrix.get_nonlocal(lp) };
        if let Ok(index) = DynamicInterruptVectorNum::try_from(vector) {
            let index: usize = index.into();
            table[index].is_none()
        } else {
            false
        }
    }

    fn dynamic_vectors_used(&self, lp: LpId) -> u64 {
        let used = unsafe { self.dynamic_vectors_used.get_nonlocal(lp) };
        *used
    }
}
