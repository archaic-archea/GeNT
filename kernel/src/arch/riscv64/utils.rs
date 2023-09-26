pub fn slow() {
    unsafe {
        core::arch::riscv64::pause();
        core::arch::asm!("wfi")
    };
}