pub struct IvrsIndex {
    ivhd_indices: Vec<IvhdIndex>,
}

pub struct IvhdIndex {
    addr: PhysicalAddress,
    device_entry_addrs: Vec<PhysicalAddress>,
    ivmd_addrs: Vec<PhysicalAddress>,
}
