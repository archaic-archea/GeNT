pub const xorshift = @import("xorshift.zig");

pub fn bitmask(bit_num: usize) usize {
    var cur_bitmask: usize = 0;
    for (0..bit_num) |_| {
        cur_bitmask = cur_bitmask << 1;
        cur_bitmask += 1;
    }

    return cur_bitmask;
}
