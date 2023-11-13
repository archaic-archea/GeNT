const sbi = @import("./sbi.zig");

pub fn set_timer(time_val: u64) !void {
    try sbi.sbi_call(0x54494D45, 0, .{ .arg0 = time_val });
}
