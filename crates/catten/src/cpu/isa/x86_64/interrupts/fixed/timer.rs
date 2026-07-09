use crate::timers::TIMER_QUEUES;

unsafe extern "custom" {
    pub unsafe fn isr_lapic_timer();
}
core::arch::global_asm!(include_str!("timer.asm"));

#[unsafe(no_mangle)]
pub extern "C" fn process_events() {
    if let Ok(mut timer_queue) = TIMER_QUEUES.try_get_mut() {
        timer_queue.process_events();
    }
}
