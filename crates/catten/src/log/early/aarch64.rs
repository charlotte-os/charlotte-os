//! # Early Log Console Backend for AArch64
//!
//! AArch64 has no architectural console. The nearest thing to one is the ARM PrimeCell PL011 that
//! QEMU's `virt` machine and the SBSA platforms place at physical `0x0900_0000`, so that is what
//! this backend drives.
//!
//! Unlike its siblings it cannot simply write to a fixed address. x86-64 reaches COM1 through the
//! I/O space, which the MMU does not govern, and RISC-V hands the byte to the SEE, which does its
//! own addressing; on AArch64 a device register is ordinary memory and has to be mapped before it
//! can be touched. Limine's higher half direct map covers usable RAM and the regions named in the
//! memory map, but not arbitrary MMIO: on the `virt` machine the gibibyte holding the PL011 has no
//! level 1 descriptor at all, and the kernel's own paging code is still unimplemented this early.
//! So this module installs the one descriptor it needs by hand, directly into the translation
//! tables Limine left live, and only ever into a slot that is currently unused.

use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::environment::boot_protocol::limine::HHDM_REQUEST;

/// Physical base of the platform's first PL011.
const PL011_BASE: usize = 0x0900_0000;

/* Register offsets from the base address. Every one of them is thirty-two bits wide. */
const DR: usize = 0x000;
const FR: usize = 0x018;
const CR: usize = 0x030;
const IMSC: usize = 0x038;
const PERIPH_ID0: usize = 0xfe0;
const PERIPH_ID1: usize = 0xfe4;
const PCELL_ID0: usize = 0xff0;
const PCELL_ID1: usize = 0xff4;
const PCELL_ID2: usize = 0xff8;
const PCELL_ID3: usize = 0xffc;

/// The value every ARM PrimeCell reports in the low bytes of its four PCellID registers.
const PRIMECELL_ID: u32 = 0xb105_f00d;
/// The PL011's part number, held in PeriphID0 and the low nibble of PeriphID1.
const PL011_PART_NUMBER: u32 = 0x011;

/// Flag register: the transmit FIFO is full and will drop anything written to the data register.
const FR_TXFF: u32 = 1 << 5;
/// Control register: the UART as a whole is enabled.
const CR_UARTEN: u32 = 1 << 0;
/// Control register: the transmitter is enabled.
const CR_TXE: u32 = 1 << 8;

/// How many times to poll the flag register before giving a byte up for lost, so that a far end
/// which never drains the FIFO cannot hang the kernel.
const TX_POLL_LIMIT: u32 = 100_000;

/// Virtual base the PL011 was found at, or zero if it could not be reached.
static BASE: AtomicUsize = AtomicUsize::new(0);

/// Reports whether `vaddr` has a stage 1 translation that EL1 may write through.
///
/// This is the only way to ask the question without risking the abort it is trying to avoid.
fn is_writable(vaddr: usize) -> bool {
    let par_el1: u64;

    // SAFETY: `AT` only queries the translation tables; it never accesses the address itself, so
    // an unmapped or read-only `vaddr` sets PAR_EL1.F rather than faulting.
    unsafe {
        asm!(
            // Address translation, stage 1, EL1, write permissions.
            "at s1e1w, {vaddr}",
            // Weakly ordered ISA is weakly ordered; the result is not visible without this.
            "isb",
            "mrs {par}, par_el1",
            vaddr = in(reg) vaddr as u64,
            par = out(reg) par_el1,
            options(nostack, preserves_flags),
        );
    }

    // PAR_EL1.F is set when the translation faulted.
    par_el1 & 1 == 0
}

/// Bits [47:12] of a table or block descriptor, and of `TTBR1_EL1`, that hold an output address.
const OUTPUT_ADDR_MASK: u64 = 0x0000_ffff_ffff_f000;

/// Installs a 1 GiB Device block covering physical `[0, 1 GiB)` at the same offset in the higher
/// half direct map, returning the table slot it filled so the caller can put it back.
///
/// The descriptor is written straight into the tables `TTBR1_EL1` already points at, because there
/// is no allocator, no VMM and no spare page this early. That is only sound because the slot it
/// fills is empty: the walk gives up the moment anything is found there.
fn map_low_gib_as_device(hhdm: usize) -> Option<*mut u64> {
    /// `TCR_EL1.TG1` encoding for the 4 KiB granule.
    const TG1_4KIB: u64 = 0b10;
    /// `T1SZ` for a 48-bit kernel region, the only size whose walk starts at level 0.
    const T1SZ_48BIT: u64 = 16;

    const DESC_BLOCK: u64 = 0b01;
    const DESC_TABLE: u64 = 0b11;
    /// Access flag; without it the first touch takes an access flag fault.
    const DESC_AF: u64 = 1 << 10;
    const DESC_PXN: u64 = 1 << 53;
    const DESC_UXN: u64 = 1 << 54;

    let (tcr_el1, ttbr1_el1, mair_el1): (u64, u64, u64);

    // SAFETY: three system register reads with no side effects.
    unsafe {
        asm!(
            "mrs {tcr}, tcr_el1",
            "mrs {ttbr1}, ttbr1_el1",
            "mrs {mair}, mair_el1",
            tcr = out(reg) tcr_el1,
            ttbr1 = out(reg) ttbr1_el1,
            mair = out(reg) mair_el1,
            options(nomem, nostack, preserves_flags),
        );
    }

    if (tcr_el1 >> 30) & 0b11 != TG1_4KIB || (tcr_el1 >> 16) & 0x3f != T1SZ_48BIT {
        return None;
    }

    // A MAIR attribute of 0x00 is Device-nGnRnE, the only memory type ordered strictly enough that
    // consecutive register writes reach the UART in the order they were issued.
    let attr_index = match (0..8).find(|i| (mair_el1 >> (i * 8)) & 0xff == 0) {
        Some(index) => index as u64,
        None => return None,
    };

    // Physical zero sits at index zero of every level, in the same level 0 table that maps RAM at
    // HHDM + 1 GiB, so that entry is known to exist and only the level 1 slot has to be filled.
    let l0_table = (hhdm + (ttbr1_el1 & OUTPUT_ADDR_MASK) as usize) as *const u64;

    // SAFETY: `TTBR1_EL1` names a live table and RAM is reachable through the direct map, which is
    // how the kernel is executing at all.
    let l0_entry = unsafe { core::ptr::read_volatile(l0_table) };
    if l0_entry & 0b11 != DESC_TABLE {
        return None;
    }

    let l1_entry = (hhdm + (l0_entry & OUTPUT_ADDR_MASK) as usize) as *mut u64;
    // SAFETY: as above; the level 0 entry was just confirmed to point at a table.
    if unsafe { core::ptr::read_volatile(l1_entry) } & 1 != 0 {
        return None;
    }

    let descriptor = DESC_BLOCK | (attr_index << 2) | DESC_AF | DESC_PXN | DESC_UXN;

    // SAFETY: the slot was just read as invalid, so nothing is being replaced.
    unsafe {
        write_descriptor(l1_entry, descriptor);
    }

    Some(l1_entry)
}

/// Publishes a translation table entry to the table walker.
///
/// # Safety
///
/// `slot` must be a live translation table entry that the caller is entitled to replace.
unsafe fn write_descriptor(slot: *mut u64, descriptor: u64) {
    unsafe {
        core::ptr::write_volatile(slot, descriptor);
        asm!(
            "dsb ishst",
            "tlbi vmalle1is",
            "dsb ish",
            "isb",
            options(nostack, preserves_flags),
        );
    }
}

/// Reports whether `base` is really a PL011 rather than RAM or some other peripheral.
///
/// `0x0900_0000` only holds a PL011 on QEMU's `virt` machine and the SBSA platforms. Elsewhere it
/// may be a different device, or plain DRAM that the bootloader mapped, and writing UART registers
/// into either would be silent corruption. Every ARM PrimeCell identifies itself in its last eight
/// registers, so those are read — and nothing is written — until the part answers for itself.
fn is_pl011(base: usize) -> bool {
    // SAFETY: the caller has established that `base` translates; these are read-only accesses to
    // the identification registers at the top of the peripheral's 4 KiB window.
    let id_byte = |offset| unsafe { core::ptr::read_volatile(reg(base, offset)) } & 0xff;

    let pcell_id = id_byte(PCELL_ID0)
        | (id_byte(PCELL_ID1) << 8)
        | (id_byte(PCELL_ID2) << 16)
        | (id_byte(PCELL_ID3) << 24);
    let part_number = id_byte(PERIPH_ID0) | ((id_byte(PERIPH_ID1) & 0x0f) << 8);

    pcell_id == PRIMECELL_ID && part_number == PL011_PART_NUMBER
}

/// Returns the first candidate address the PL011 answers at, or zero if it is out of reach.
///
/// The higher half direct map is tried first, then the same address once the missing gibibyte has
/// been mapped, and finally the bare physical address for a bootloader that leaves an identity map
/// installed in TTBR0_EL1.
///
/// The offset is taken straight from Limine rather than through
/// [`HHDM_BASE`](crate::memory::HHDM_BASE), whose `From<usize>` conversion applies x86-64
/// canonical sign extension and so truncates AArch64's `0xffff_0000_0000_0000` TTBR1 base to zero.
fn find_base() -> usize {
    let hhdm = HHDM_REQUEST.response().map_or(0, |response| response.offset as usize);

    if hhdm != 0 {
        let candidate = hhdm + PL011_BASE;

        if is_writable(candidate) {
            // Already mapped, so this gibibyte is something the bootloader put there. It is only a
            // console if the part says so; on a board whose DRAM starts at zero it is just RAM.
            if is_pl011(candidate) {
                return candidate;
            }
        } else if let Some(slot) = map_low_gib_as_device(hhdm) {
            if is_writable(candidate) && is_pl011(candidate) {
                return candidate;
            }

            // SAFETY: `slot` was invalid before this module filled it, so putting it back is a
            // restore rather than an unmapping of someone else's memory. Leaving a Device alias of
            // a gibibyte nobody asked for would risk mismatched attributes against another mapping.
            unsafe {
                write_descriptor(slot, 0);
            }
        }
    }

    if is_writable(PL011_BASE) && is_pl011(PL011_BASE) {
        return PL011_BASE;
    }

    0
}

fn reg(base: usize, offset: usize) -> *mut u32 {
    (base + offset) as *mut u32
}

pub(super) fn init() {
    let base = find_base();
    BASE.store(base, Ordering::Relaxed);

    if base == 0 {
        return;
    }

    // The firmware that loaded the kernel has already brought the UART up at whatever baud rate
    // its own console used, and the divisors that would reproduce it depend on a UARTCLK this
    // code has no way to learn. So the line settings are left exactly as they were found and only
    // the two things that matter here are asserted: that nothing will raise an interrupt, and
    // that the transmitter is on.
    unsafe {
        core::ptr::write_volatile(reg(base, IMSC), 0);
        let cr = core::ptr::read_volatile(reg(base, CR));
        core::ptr::write_volatile(reg(base, CR), cr | CR_UARTEN | CR_TXE);
    }
}

pub(super) fn write_byte(byte: u8) {
    let base = BASE.load(Ordering::Relaxed);
    if base == 0 {
        return;
    }

    unsafe {
        for _ in 0..TX_POLL_LIMIT {
            if core::ptr::read_volatile(reg(base, FR)) & FR_TXFF == 0 {
                core::ptr::write_volatile(reg(base, DR), byte as u32);
                return;
            }
            core::hint::spin_loop();
        }
    }
}
