const sbi = @import("./sbi.zig");

pub fn send_ipi(hart_mask: usize, hart_mask_base: usize) !void {
    try sbi.sbi_call(0x735049, 0x0, .{ .arg0 = hart_mask, .arg1 = hart_mask_base });
}
