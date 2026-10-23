use crate::memory::{PhysicalAddress, VirtualAddress};

pub mod physical_mapping;
pub mod virtual_mapping;

pub struct PcieBarWindow {
    virt_base: VirtualAddress,
    phys_base: PhysicalAddress,
    size: usize,
}
