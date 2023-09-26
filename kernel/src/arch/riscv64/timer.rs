pub fn get_timer() -> u128 {
    let val: u64;

    unsafe {
        core::arch::asm!(
            "rdtime {reg}",
            reg = out(reg) val
        )
    }

    val as u128
}