use crate::memory::VirtualAddress;

/// Read an 8-bit value from the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to an 8-bit register in a PCIe ECAM.
pub unsafe fn config_read8(addr: VirtualAddress) -> u8 {
    let mut ret: u8;
    unsafe {
        core::arch::asm!(
            "mov {}, byte ptr [{}]",
            out(reg) ret,
            in(reg) addr,
        );
        ret
    }
}

/// Read a 16-bit value from the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to a 16-bit register in a PCIe ECAM.
pub unsafe fn config_read16(addr: VirtualAddress) -> u16 {
    let mut ret: u16;
    unsafe {
        core::arch::asm!(
            "mov {}, word ptr [{}]",
            out(reg) ret,
            in(reg) addr,
        );
        ret
    }
}

/// Read a 32-bit value from the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to a 32-bit register in a PCIe ECAM.
pub unsafe fn config_read32(addr: VirtualAddress) -> u32 {
    let mut ret: u32;
    unsafe {
        core::arch::asm!(
            "mov {}, dword ptr [{}]",
            out(reg) ret,
            in(reg) addr,
        );
        ret
    }
}

/// Write an 8-bit value to the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to an 8-bit writable register in a PCIe
/// ECAM.
pub unsafe fn config_write8(addr: VirtualAddress, mut value: u8) {
    unsafe {
        core::arch::asm!(
            "mov byte ptr [{addr}], {value}",
            // The load needs to be kept after the store as its purpose is to ensure the write has completed on the PCIe fabric.
            "lfence",
            // Read back the value to ensure the write has completed before continuing.
            "mov {value}, byte ptr [{addr}]",
            /* An additional fence is not needed because the read-back ensures the write has completed
            and no loads or stores can be reordered before the read-back completes due to x86_64 guaranteeing TSO (Total Store Order) */
            addr = in(reg) addr,
            value = inlateout(reg) value,
        );
    }
}

/// Write a 16-bit value to the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to a 16-bit writable register in a PCIe
/// ECAM.
pub unsafe fn config_write16(addr: VirtualAddress, mut value: u16) {
    unsafe {
        core::arch::asm!(
            "mov word ptr [{addr}], {value}",
            // The load needs to be kept after the store as its purpose is to ensure the write has completed on the PCIe fabric.
            "lfence",
            // Read back the value to ensure the write has completed before continuing.
            "mov {value}, word ptr [{addr}]",
            /* An additional fence is not needed because the read-back ensures the write has completed
            and no loads or stores can be reordered before the read-back completes due to x86_64 guaranteeing TSO (Total Store Order) */
            addr = in(reg) addr,
            value = inlateout(reg) value,
        );
    }
}

/// Write a 32-bit value to the PCIe configuration space at the given address.
///
/// Safety: The caller must ensure that the address points to a 32-bit writable register in a PCIe
/// ECAM.
pub unsafe fn config_write32(addr: VirtualAddress, mut value: u32) {
    unsafe {
        core::arch::asm!(
            "mov dword ptr [{addr}], {value}",
            // The load needs to be kept after the store as its purpose is to ensure the write has completed on the PCIe fabric.
            "lfence",
            // Read back the value to ensure the write has completed before continuing.
            "mov {value}, dword ptr [{addr}]",
            /* An additional fence is not needed because the read-back ensures the write has completed
            and no loads or stores can be reordered before the read-back completes due to x86_64 guaranteeing TSO (Total Store Order) */
            addr = in(reg) addr,
            value = inlateout(reg) value,
        );
    }
}
