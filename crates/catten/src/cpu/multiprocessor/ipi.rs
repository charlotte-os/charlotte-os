//! # Inter-Processor Interrupts (IPIs)
//!
//! The Catten IPI protocol is designed to work using remote procedure calls (RPCs).
//! This allows for a flexible and extensible way to send IPIs between processors.
//! The protocol uses unicast IPIs exculusively to avoid having to overcomplicate the
//! implementation which is also kept as architecture indepent as possible.

use alloc::format;
use alloc::vec::Vec;

use concurrent_queue::ConcurrentQueue;

use crate::cpu::isa::constants::interrupt_vectors::IPI_DOORBELL_VECTOR;
use crate::cpu::isa::interface::interrupts::LocalIntCtlrIfce;
use crate::cpu::isa::interrupts::LocalIntCtlr;
use crate::cpu::isa::lp::LpId;
use crate::cpu::isa::lp::ops::get_lp_id;
use crate::cpu::isa::memory::tlb;
use crate::cpu::multiprocessor::get_lp_count;
use crate::cpu::scheduler::system_scheduler::SYSTEM_SCHEDULER;
use crate::memory::linear::VirtualAddress;
use crate::memory::{AddressSpaceId, KERNEL_ASID, get_current_asid};

pub static IPI_CMD_QUEUES: spin::LazyLock<IpiCmdQueues> = spin::LazyLock::new(IpiCmdQueues::new);

#[inline(always)]
pub fn send_doorbell_ipi(target_lp: LpId) {
    LocalIntCtlr::send_unicast_ipi(target_lp, IPI_DOORBELL_VECTOR)
        .expect(&format!("Failed to send an IPI from LP {} to LP {target_lp}", get_lp_id()));
}

#[inline(always)]
pub fn send_multicast_doorbell_ipi(lps: &[LpId]) {
    for &lp in lps {
        send_doorbell_ipi(lp);
    }
}

#[derive(Debug)]
pub struct IpiCmdQueues {
    queues: Vec<ConcurrentQueue<IpiRpc>>,
}

impl IpiCmdQueues {
    pub fn new() -> Self {
        Self {
            queues: (0..get_lp_count()).map(|_| ConcurrentQueue::unbounded()).collect::<Vec<_>>(),
        }
    }

    pub fn push_to(&self, target_lp: usize, ipi: IpiRpc) {
        self.queues[target_lp].push(ipi).expect("Failed to push IPI command to target LP");
    }

    pub fn push_to_multiple(&self, lps: &[LpId], ipi: IpiRpc) {
        for &lp in lps {
            self.queues[lp as usize]
                .push(ipi.clone())
                .expect("Failed to push IPI command to target LP");
        }
    }

    pub fn pop_local(&self) -> Option<IpiRpc> {
        let lp_id = get_lp_id();
        self.queues[lp_id as usize].pop().ok()
    }
}

#[derive(Debug, Clone)]
pub enum IpiRpc {
    VMemInval(AddressSpaceId, VirtualAddress, usize),
    Wakeup,
}

impl IpiRpc {
    pub fn process(self) {
        match self {
            IpiRpc::VMemInval(asid, base, size) => {
                if get_current_asid() == asid || asid == KERNEL_ASID {
                    tlb::inval_range(base, size);
                }
            }
            IpiRpc::Wakeup => {
                SYSTEM_SCHEDULER.read().get_lp_scheduler().lock().set_ctx_switch_pending();
            }
        }
    }
}
