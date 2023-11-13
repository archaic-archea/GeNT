/// Read the stime timer
pub fn time() usize {
    return asm volatile (
        \\ csrr t0, time
        : [ret] "={t0}" (-> usize),
        :
        : "t0"
    );
}
