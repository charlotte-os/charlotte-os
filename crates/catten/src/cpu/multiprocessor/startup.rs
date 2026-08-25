use limine::mp::MP_FLAG_X2APIC;
use spin::{LazyLock, RwLock};

use crate::environment::boot_protocol::limine::MP_REQUEST;
use crate::{ap_main, early_logln, logln};

pub(super) static LP_COUNT: LazyLock<RwLock<u32>> = LazyLock::new(|| {
    RwLock::new({
        if let Some(mp_res) = MP_REQUEST.response() {
            mp_res.cpus().len() as u32
        } else {
            panic!("Limine was not able to start the secondary logical processors!")
        }
    })
});

#[derive(Debug)]
pub enum MpError {
    SecondaryLpStartupFailed,
}

pub fn start_secondary_lps() -> Result<(), MpError> {
    logln!("Starting Secondary LPs...");
    if let Some(res) = MP_REQUEST.response() {
        logln!("Obtained multiprocessor response from Limine");
        cfg_select! {
            target_arch = "x86_64" => {
                if res.flags & MP_FLAG_X2APIC as u32 != 0 {
                    logln!("Limine has set all LAPICs to x2APIC mode.")
                } else {
                    panic!("Processor not supported: x2APIC mode is not available.");
                }
            }
            _ => { /* Non-x86_64 ISAs require no special secondary processor startup handling */ }
        }
        let lps = res.cpus();
        for lp in lps {
            logln!("Writing entry point address for LP {}", (lp.processor_id));
            lp.bootstrap(ap_main, 0);
        }
        Ok(())
    } else {
        Err(MpError::SecondaryLpStartupFailed)
    }
}

use core::sync::atomic::{AtomicU32, Ordering};

use crate::cpu::isa::lp::ops::*;

pub static ID_COUNTER: AtomicU32 = AtomicU32::new(0);

pub unsafe fn assign_id() {
    let lp_id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    store_lp_id(lp_id);
    if lp_id == 0 {
        early_logln!(
            "Logical Processor with local interrupt controller ID = {} has been designated LP {}.",
            (get_lic_id()),
            (get_lp_id())
        );
    } else {
        logln!(
            "Logical Processor with local interrupt controller ID = {} has been designated LP {}.",
            (get_lic_id()),
            (get_lp_id())
        );
    }
}
