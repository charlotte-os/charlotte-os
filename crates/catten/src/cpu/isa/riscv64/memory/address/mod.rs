use spin::LazyLock;

pub use crate::cpu::isa::common::memory::address::*;
use crate::cpu::isa::interface::system_info::CpuInfoIfce;
use crate::cpu::isa::riscv64::system_info::CpuInfo;

pub static PADDR_SIG_BITS: LazyLock<u8> = LazyLock::new(CpuInfo::get_paddr_sig_bits);
pub static PADDR_MASK: LazyLock<usize> = LazyLock::new(|| (1 << *PADDR_SIG_BITS as usize) - 1);
pub static VADDR_SIG_BITS: LazyLock<u8> = LazyLock::new(CpuInfo::get_vaddr_sig_bits);
pub static VADDR_MASK: LazyLock<usize> = LazyLock::new(|| (1 << *VADDR_SIG_BITS as usize) - 1);
