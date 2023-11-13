const sbi = @import("./sbi.zig");

pub const SbiVersion = packed struct {
    minor: u24 = 0,
    major: u7 = 0,
    _res: u1 = 0,
};

pub fn version() SbiVersion {
    const raw_version: u32 = @intCast(sbi.sbi_call(0x10, 0x0, .{}) catch |err| {
        switch (err) {
            else => unreachable,
        }
    });

    return @bitCast(raw_version);
}

pub fn impl_id() usize {
    return sbi.sbi_call(0x10, 0x1, .{}) catch |err| {
        switch (err) {
            else => unreachable,
        }
    };
}

pub fn impl_version() usize {
    return sbi.sbi_call(0x10, 0x2, .{}) catch |err| {
        switch (err) {
            else => unreachable,
        }
    };
}

pub fn probe_extension(eid: usize) bool {
    return sbi.sbi_call(0x10, 0x3, .{ .arg0 = eid }) catch |err| {
        switch (err) {
            else => unreachable,
        }
    } == 1;
}

pub fn vendor_id() usize {
    return sbi.sbi_call(0x10, 0x4, .{}) catch |err| {
        switch (err) {
            else => unreachable,
        }
    };
}

pub fn arch_id() usize {
    return sbi.sbi_call(0x10, 0x5, .{}) catch |err| {
        switch (err) {
            else => unreachable,
        }
    };
}

pub fn machine_impl_id() usize {
    return sbi.sbi_call(0x10, 0x6, .{}) catch |err| {
        switch (err) {
            else => unreachable,
        }
    };
}
