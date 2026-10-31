use crate::cpu::isa::interface::interrupts::LocalIntCtlrIfce;

pub mod x2apic;
pub mod xapic;

pub enum Lapic {
    Xapic(xapic::Xapic),
    X2Apic(x2apic::X2Apic),
}

impl LocalIntCtlrIfce for Lapic {
    type Error = super::Error;

    fn init_lp(&self) {
        match self {
            Lapic::Xapic(x) => {
                xapic::Xapic::init_lp();
            }
            Lapic::X2Apic(x) => {
                x.init_lp();
            }
        }
    }
}
