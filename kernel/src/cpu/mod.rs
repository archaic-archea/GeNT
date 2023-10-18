use spin::Mutex;

#[thread_local]
// The ID of the current process
pub static THREAD_CTRL_BLOCK: Mutex<ThreadCtrlBlock> = Mutex::new(
    ThreadCtrlBlock { 
        proc_id: 0 
    }
);

/// Allows for interaction and control of the processor core/thread you're hosted on
pub struct ThreadCtrlBlock {
    proc_id: u128,
    
}

impl ThreadCtrlBlock {
    pub fn proc_id(&self) -> u128 {
        self.proc_id
    }
}