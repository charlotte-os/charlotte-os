use crate::cpu::multiprocessor::ipi::IPI_CMD_QUEUES;

core::arch::global_asm!(include_str!("ipis.asm"));

unsafe extern "custom" {
    pub fn isr_ipi_doorbell();
}

#[unsafe(no_mangle)]
pub extern "C" fn ih_ipi_doorbell() {
    while let Some(ipi_rpc) = IPI_CMD_QUEUES.pop_local() {
        ipi_rpc.process();
    }
}
