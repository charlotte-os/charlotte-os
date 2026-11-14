use alloc::vec::Vec;

pub fn reap_exited_threads() {
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
        thread_table.remove(idx);
    }
}
