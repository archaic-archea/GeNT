use super::trap::stvec_trap_shim;

#[naked]
#[no_mangle]
#[link_section = ".initext"]
unsafe extern "C" fn _boot() -> ! {
    core::arch::asm!("
        csrw sie, zero
        csrci sstatus, 2
        
        .option push
        .option norelax
        lla gp, __global_pointer
        .option pop

        lla sp, __stack_top

        lla t1, {}
        csrw stvec, t1

        2:
            j kinit
    ", sym stvec_trap_shim, options(noreturn));
}