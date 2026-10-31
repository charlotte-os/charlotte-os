//! # xAPIC Local Advanced Programmable Interrupt Controller
mod id;
mod io;

use core::mem::MaybeUninit;

use spin::LazyLock;

use crate::cpu::isa::constants::interrupt_vectors::*;
use crate::cpu::isa::interface::interrupts::LocalIntCtlrIfce;
use crate::cpu::isa::interface::timers::LpTimerIfce;
use crate::cpu::isa::interrupts::Error;
use crate::cpu::isa::interrupts::lapic::xapic::io::xapic_io_write32;
use crate::cpu::isa::lp::{IntSrcDscr, LpId};
use crate::cpu::isa::timers::apic_timer::ApicTimer;
use crate::cpu::isa::x86_64::constants::xapic;
use crate::cpu::multiprocessor::spin::per_lp::PerLp;
use crate::get_lp_id;

pub static LAPICS: LazyLock<PerLp<MaybeUninit<Xapic>>> =
    LazyLock::new(|| PerLp::new(|| MaybeUninit::uninit()));

/// # Interrupt Command Register Delivery Mode
#[repr(u32)]
enum IcrDeliveryMode {
    Fixed = 0b000,
    _Smi = 0b010,
    _Nmi = 0b100,
    _Init = 0b101,
    _Startup = 0b110,
}

/// # Interrupt Command Register Destination Shorthand
#[repr(u32)]
enum IcrDestShorthand {
    NoShorthand = 0b00,
    _OnlySelf = 0b01,
    _AllIncludingSelf = 0b10,
    _AllExcludingSelf = 0b11,
}

pub struct Xapic {
    pub timer: ApicTimer,
}

impl Xapic {
    /// # Initialize the local APIC in xapic mode
    /// Ref: AMD APM 16.4.7
    fn new(timer_int_vec: <ApicTimer as LpTimerIfce>::IntDispatchNum) -> Self {
        // Set the Spurious Interrupt Vector Register (SIVR)
        const ASE_BIT_SHIFT: u32 = 8;
        const VEC_MASK: u32 = 0xff;
        let sivr_val: u32 =
            (SPURIOUS_INTERRUPT_VECTOR_NUM as u32 & VEC_MASK) | (1 << ASE_BIT_SHIFT); // APIC Software Enable
        unsafe {
            xapic_io_write32(xapic::SPURIOUS_INTERRUPT_VECTOR_REG, sivr_val);
        }
        Self::set_timer_lvt_entry(timer_int_vec, false);
        Xapic {
            timer: ApicTimer::new(timer_int_vec),
        }
    }

    pub fn record_id() {
        id::XAPIC_ID_TABLE.lock().insert(get_lp_id(), id::LapicId::get_local());
    }

    fn translate_lp_id(lp_id: LpId) -> Option<id::LapicId> {
        id::XAPIC_ID_TABLE.lock().get(&lp_id).cloned()
    }

    pub fn physical_id(lp_id: LpId) -> Option<id::PhysicalLapicId> {
        Self::translate_lp_id(lp_id).map(|id| id.physical)
    }

    fn make_icr_low(
        vector: u8,
        delivery_mode: IcrDeliveryMode,
        is_dest_logical: bool,
        level: bool,
        is_level_triggered: bool,
        dest_shorthand: IcrDestShorthand,
    ) -> u32 {
        const DELIVERY_MODE_SHIFT: u32 = 8;
        const IS_DEST_LOGICAL_SHIFT: u32 = 11;
        const LEVEL_SHIFT: u32 = 14;
        const IS_LEVEL_TRIGGERED_SHIFT: u32 = 15;
        const DEST_SHORTHAND_SHIFT: u32 = 18;

        vector as u32
            | ((delivery_mode as u32) << DELIVERY_MODE_SHIFT)
            | ((is_dest_logical as u32) << IS_DEST_LOGICAL_SHIFT)
            | ((level as u32) << LEVEL_SHIFT)
            | ((is_level_triggered as u32) << IS_LEVEL_TRIGGERED_SHIFT)
            | ((dest_shorthand as u32) << DEST_SHORTHAND_SHIFT)
    }

    pub fn set_timer_lvt_entry(
        interrupt_vector: <ApicTimer as LpTimerIfce>::IntDispatchNum,
        periodic: bool,
    ) {
        const TIMER_MODE_SHIFT: u32 = 17;
        const TIMER_MODE_PERIODIC: u32 = 0b1;
        const TIMER_MODE_ONE_SHOT: u32 = 0b0;
        const MASK_BIT_SHIFT: u32 = 16;
        const TIMER_VECTOR_MASK: u32 = 0xff;

        let timer_lvt_entry = (interrupt_vector as u32 & TIMER_VECTOR_MASK)
            | (if periodic {
                TIMER_MODE_PERIODIC
            } else {
                TIMER_MODE_ONE_SHOT
            } << TIMER_MODE_SHIFT)
                & !(1u32 << MASK_BIT_SHIFT); // Unmask the timer interrupt
        unsafe {
            xapic_io_write32(xapic::TIMER_LVTR, timer_lvt_entry);
        }
    }
}

impl LocalIntCtlrIfce for Xapic {
    type Error = Error;

    fn init_lp() {
        let lapic = Xapic::new(LAPIC_TIMER_VECTOR);
        LAPICS
            .try_get_mut()
            .expect("Failed to obtain LAPIC structure during LP init.")
            .write(lapic);
        Self::record_id();
    }

    /// Send a unicast IPI to the target logical processor
    ///
    /// Ref: Intel SDM Vol.3 12.12.10.1
    fn send_unicast_ipi(target_lp: LpId, target_vector: IntSrcDscr) -> Result<(), Error> {
        if let Some(apic_id) = Self::translate_lp_id(target_lp) {
            // Get the physical APIC ID for the target LP
            let dest = apic_id.physical;
            // Construct the ICR low dword
            let icr_low = Self::make_icr_low(
                target_vector,
                IcrDeliveryMode::Fixed,
                false,
                true,
                false,
                IcrDestShorthand::NoShorthand,
            );
            // Write to the Interrupt Command Register MSR to send the IPI
            unsafe {
                xapic_io_write32(
                    xapic::INTERRUPT_COMMAND_REGISTER_HIGH,
                    (dest as u32) << id::LapicId::XAPIC_ID_SHIFT,
                );
                xapic_io_write32(xapic::INTERRUPT_COMMAND_REGISTER_LOW, icr_low);
            }
            // Success
            Ok(())
        } else {
            Err(Error::InvalidLpId)
        }
    }

    #[unsafe(no_mangle)]
    extern "C" fn signal_eoi() {
        unsafe {
            xapic_io_write32(xapic::EOI_REGISTER, 0);
        }
    }
}
