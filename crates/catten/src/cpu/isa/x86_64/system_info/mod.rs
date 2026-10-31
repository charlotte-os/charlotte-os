use alloc::string::String;
use alloc::vec::Vec;
use core::arch::x86_64::__cpuid_count;
use core::mem::transmute;

use spin::lazylock::LazyLock;

use crate::cpu::isa::constants;
use crate::cpu::isa::interface::system_info::CpuInfoIfce;

pub static IS_CPUID_SUPPORTED: LazyLock<bool> = LazyLock::new(is_cpuid_supported);

#[inline]
fn is_cpuid_supported() -> bool {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop rax",
            "xor rax, {id_mask:r}",
            "push rax",
            "popfq",
            "pushfq",
            "pop rax",
            "and rax, {id_mask:r}",
            id_mask = in(reg) 1 << 21,
            lateout("rax") ret,
        );
        ret != 0
    }
}

pub enum IsaExtension {
    /* indicates support for 5-level paging i.e. 57 bit linear addresses */
    La57,
    /* indicates support for `invlpgb` (Invalidate Page with Broadcast) and `tlbsync`
     * (TLB shootdown synchronization after `invlpgb`) */
    Invlpgb,
    InvariantTsc,
    /* TSC_AUX MSR and RDPID instruction */
    Rdpid,
    TscDeadline,
}

pub struct CpuInfo;

impl CpuInfoIfce for CpuInfo {
    type IsaExtension = IsaExtension;
    type Model = String;
    type Vendor = String;

    fn get_vendor() -> Self::Vendor {
        unsafe {
            let vendor_string_raw = __cpuid_count(0, 0);
            let utf8 = Vec::from(transmute::<[u32; 3], [u8; 12]>([
                vendor_string_raw.ebx,
                vendor_string_raw.edx,
                vendor_string_raw.ecx,
            ]));
            String::from_utf8(utf8).unwrap()
        }
    }

    fn get_model() -> Self::Model {
        unsafe {
            let cpuid_results = [
                __cpuid_count(0x80000002, 0),
                __cpuid_count(0x80000003, 0),
                __cpuid_count(0x80000004, 0),
            ];
            let utf8 = Vec::from(transmute::<[u32; 12], [u8; 48]>([
                cpuid_results[0].eax,
                cpuid_results[0].ebx,
                cpuid_results[0].ecx,
                cpuid_results[0].edx,
                cpuid_results[1].eax,
                cpuid_results[1].ebx,
                cpuid_results[1].ecx,
                cpuid_results[1].edx,
                cpuid_results[2].eax,
                cpuid_results[2].ebx,
                cpuid_results[2].ecx,
                cpuid_results[2].edx,
            ]));
            // Convert the byte vector to a String, assuming it is valid UTF-8
            // Note: This is safe because the cpuid results are guaranteed to be valid UTF-8
            // as per the AMD64 Architecture Programmer's Manual.
            String::from_utf8(utf8).unwrap().trim_end_matches("\0").into()
        }
    }

    fn get_paddr_sig_bits() -> u8 {
        let cpuid_result = __cpuid_count(0x80000008, 0);
        cpuid_result.eax as u8
    }

    fn get_vaddr_sig_bits() -> u8 {
        if Self::is_extension_supported(IsaExtension::La57) {
            let mut cr4: u64;
            unsafe {
                core::arch::asm! {
                    "mov {}, cr4",
                    out(reg) cr4
                }
            }
            if cr4 & constants::control_regs::CR4_LA57_ENABLE_BIT != 0 {
                57
            } else {
                48
            }
        } else {
            48
        }
    }

    fn is_extension_supported(extension: Self::IsaExtension) -> bool {
        if *IS_CPUID_SUPPORTED == false {
            panic!(
                "The current x86-64 processor does not support the CPUID instruction which is \
                 required by the Catten kernel."
            )
        }
        match extension {
            IsaExtension::La57 => {
                let cpuid_result = __cpuid_count(0x0000_0007, 0);
                (cpuid_result.ecx & 1 << 16) != 0
            }
            IsaExtension::Invlpgb => {
                let cpuid_result = __cpuid_count(0x8000_0008, 0);
                (cpuid_result.ebx & 1 << 5) != 0
            }
            IsaExtension::InvariantTsc => {
                let feat_ext = __cpuid_count(0x80000007, 0);
                (feat_ext.edx & (1 << 8)) != 0
            }
            IsaExtension::Rdpid => {
                let cpuid_result = __cpuid_count(0x0000_0007, 0);
                (cpuid_result.ecx & 1 << 22) != 0
            }
            IsaExtension::TscDeadline => {
                let cpuid_result = __cpuid_count(0x0000_0007, 0);
                (cpuid_result.ecx & 1 << 24) != 0
            }
        }
    }
}
