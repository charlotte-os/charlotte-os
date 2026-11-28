//! # Multi-Processor Management
pub mod cpu_topology;
mod int_state;
pub mod ipi;
pub mod per_lp;
pub mod spin;
pub mod startup;

#[inline]
pub fn get_lp_count() -> u32 {
    *(startup::LP_COUNT).read()
}
#[inline]
pub fn get_core_count() -> u32 {
    todo!()
}
