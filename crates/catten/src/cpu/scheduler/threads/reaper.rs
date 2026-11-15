use alloc::vec::Vec;

use crate::klib::time::duration::ExtDuration;

pub extern "C" fn run_reaper() {
    loop {
        crate::cpu::scheduler::sleep(ExtDuration::from_secs(60));
        reap_exited_threads();
    }
}

fn reap_exited_threads() {
    let mut thread_table = crate::cpu::scheduler::threads::THREAD_DIRECTORY.write();
    let mut rm_list = Vec::new();
    for (idx, elem) in thread_table.iter().enumerate() {
        if let Some(thread) = elem {
            if let crate::cpu::scheduler::threads::ThreadState::Exited = thread.state {
                rm_list.push(idx);
            }
        }
    }
    for idx in rm_list {
        crate::log::logln!("[Reaper] Reclaiming resources from exited thread with ID = {idx}.");
        let _ = thread_table.remove(idx);
    }
}
