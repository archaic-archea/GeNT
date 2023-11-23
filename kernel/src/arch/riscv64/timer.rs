use core::sync::atomic::AtomicUsize;

pub(super) static FREQ: AtomicUsize = AtomicUsize::new(0);

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

pub fn set_timer(val: u128) {
    sbi::timer::set_timer(val as u64).unwrap();
}

pub fn ticks_from_ms(ms: usize) -> u128 {
    let freq = FREQ.load(core::sync::atomic::Ordering::Relaxed) / 1000;
    let freq = freq as u128;
    freq * ms as u128
}