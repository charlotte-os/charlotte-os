use crate::cpu::isa::interface::system_info::CpuInfoIfce;

#[derive(Debug)]
/// RISCV64-specific ISA extensions
/// These are found by checking the `misa` CSR, which is a bitfield where each bit corresponds to a
/// specific extension.
/// RVA22 or later is required so those extensions are assumed to be present and not included here.
///
/// The bit positions follow `misa`'s letter encoding, where bit 0 is 'A' and bit 25 is 'Z'.
pub enum Riscv64IsaExtension {
    Atomic = 1 << 0,                           // A
    BitManipulation = 1 << 1,                  // B
    Compressed = 1 << 2,                       // C
    DoublePrecisionFloatingPoint = 1 << 3,     // D
    Embedded = 1 << 4,                         // E
    SinglePrecisionFloatingPoint = 1 << 5,     // F
    Hypervisor = 1 << 7,                       // H
    BaseInteger = 1 << 8,                      // I
    MultiplyDivide = 1 << 12,                  // M
    QuadruplePrecisionFloatingPoint = 1 << 16, // Q
    SupervisorMode = 1 << 18,                  // S
    UserMode = 1 << 20,                        // U
    Vector = 1 << 21,                          // V
    NonStandardExtensions = 1 << 23,           // X
}

/// This is the value of the `mvendorid` CSR, which is a 64-bit value that identifies the vendor of
/// the CPU.
pub type Riscv64Vendor = u64;

#[derive(Debug)]
pub struct Riscv64Model {
    pub marchid: u64,
    pub mimpid: u64,
}

impl core::fmt::Display for Riscv64Model {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "marchid {:#018x}, mimpid {:#018x}", self.marchid, self.mimpid)
    }
}

pub struct Riscv64CpuInfo;

pub type CpuInfo = Riscv64CpuInfo;

impl CpuInfoIfce for Riscv64CpuInfo {
    type IsaExtension = Riscv64IsaExtension;
    type Model = Riscv64Model;
    type Vendor = Riscv64Vendor;

    fn get_vendor() -> Self::Vendor {
        todo!(
            "Read mvendorid through the SBI Base extension once the SBI call wrapper in \
             environment::riscv_sbi exists."
        )
    }

    fn get_model() -> Self::Model {
        todo!(
            "Read marchid and mimpid through the SBI Base extension once the SBI call wrapper in \
             environment::riscv_sbi exists."
        )
    }

    fn get_vaddr_sig_bits() -> u8 {
        todo!("Get this from Limine or the `satp` CSR, whichever is easier.")
    }

    fn get_paddr_sig_bits() -> u8 {
        // This is the only supported number of physical address bits for all currently specified
        // 64-bit RISC-V paging modes.
        56u8
    }

    fn is_extension_supported(_extension: Self::IsaExtension) -> bool {
        todo!(
            "Get this information per LP from the RHCT ACPI table or the DT CPU nodes, whichever \
             is present in the system."
        )
    }
}
