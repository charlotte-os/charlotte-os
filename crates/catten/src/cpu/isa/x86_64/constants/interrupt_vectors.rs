use hashbrown::HashSet;
use spin::LazyLock;

pub const DYN_VECS_PER_LP: u8 = 220;
#[unsafe(no_mangle)]
pub static DYN_VEC_START_OFFSET: u8 = 35;

pub const EXCEPTION_VECTOR_RANGE: core::ops::RangeInclusive<u8> = 0..=31;
pub const LAPIC_TIMER_VECTOR: u8 = 32;
pub const IPI_DOORBELL_VECTOR: u8 = 33;
pub const SPURIOUS_INTERRUPT_VECTOR_NUM: u8 = 255;

pub static FIXED_INTERRUPT_VECTORS: LazyLock<HashSet<u8>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    for i in EXCEPTION_VECTOR_RANGE {
        set.insert(i);
    }
    set.insert(LAPIC_TIMER_VECTOR);
    set.insert(IPI_DOORBELL_VECTOR);
    set.insert(SPURIOUS_INTERRUPT_VECTOR_NUM);
    set
});
