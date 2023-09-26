use spin::Mutex;

#[thread_local]
// The ID of the current process
pub static PROC_ID: Mutex<u128> = Mutex::new(0);
