const csr = @import("./csr.zig");

pub fn time() usize {
    return csr.time();
}
