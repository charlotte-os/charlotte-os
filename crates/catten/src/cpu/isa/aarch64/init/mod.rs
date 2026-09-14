use super::interrupts::load_evt;
use crate::cpu::isa::interface::init::InitInterface;
use crate::{early_logln, logln};

pub struct IsaInitializer;

#[derive(Debug)]
pub enum Error {
    // Error type for the aarch64 architecture
}

impl InitInterface for IsaInitializer {
    type Error = Error;

    #[inline(always)]
    fn init_bsp() -> Result<(), Self::Error> {
        // Initialization code for the aarch64 architecture
        early_logln!("Performing Aarch64 ISA specific initialization...");
        // Setup the exception vector table
        early_logln!("Loading the exception vector table on the BSP");
        load_evt();
        early_logln!("Exception vector table loaded on the BSP");

        early_logln!("Aarch64 ISA specific initialization complete!");
        Ok(())
    }

    fn init_ap() -> Result<(), Self::Error> {
        // Initialization code for the aarch64 architecture
        logln!("Performing Aarch64 ISA specific initialization...");
        // Setup the exception vector table
        logln!("Loading the exception vector table on the AP");
        load_evt();
        logln!("Exception vector table loaded on the AP");

        logln!("Aarch64 ISA specific initialization complete!");
        Ok(())
    }

    fn deinit() -> Result<(), Self::Error> {
        // Deinitialization code for the aarch64 architecture
        Ok(())
    }
}
