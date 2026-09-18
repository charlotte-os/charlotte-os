use hashbrown::HashSet;
use spin::LazyLock;

use crate::cpu::multiprocessor::spin::rwlock::RwLock;

pub static ACPI_CLAIMED_IO_REGIONS: LazyLock<RwLock<HashSet<IoRegion>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

pub type IoPortAddr = u16;
pub type IoUSize = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IoRegion {
    pub base: IoPortAddr,
    pub len: IoUSize,
}

impl IoRegion {
    pub fn contains(&self, addr: IoPortAddr) -> bool {
        addr >= self.base && addr < (self.base + self.len)
    }

    pub fn overlaps(&self, other: IoRegion) -> bool {
        // To not overlap this region must be completely before or after the other region.
        self.base + self.len < other.base // before
        || self.base > other.base + other.len // after
    }
}

pub fn is_acpi_claimed(addr: IoPortAddr) -> bool {
    let regions = ACPI_CLAIMED_IO_REGIONS.read();
    for region in regions.iter() {
        if region.contains(addr) {
            return true;
        }
    }
    false
}

pub fn overlaps_acpi_claimed(region: IoRegion) -> bool {
    let regions = ACPI_CLAIMED_IO_REGIONS.read();
    for claimed in regions.iter() {
        if region.overlaps(*claimed) {
            return true;
        }
    }
    false
}
