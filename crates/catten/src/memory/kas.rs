use crate::memory::{AddressSpace, AddressSpaceInterface};

pub fn create_kernel_address_space() -> AddressSpace {
    let mut kas = AddressSpace::get_current().clone();
    kas
}
