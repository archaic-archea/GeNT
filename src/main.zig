const std = @import("std");
const arch = @import("arch/arch.zig");
const locks = @import("lock.zig");
const kstdout = @import("kstdout.zig");
const alloc = @import("alloc/alloc.zig");
const crypto = @import("crypto/crypto.zig");

var UART_LOCK: locks.Mutex(*volatile u8) = .{
    .val = @ptrFromInt(0x1000_0000),
};

fn kmain() !noreturn {
    var kernout = kstdout.Kstdout{};
    const kout = kernout.writer();

    const sbi_version = arch.mod.sbi.base.version();

    try kout.print("SBI Version: {}.{}\n", .{ sbi_version.major, sbi_version.minor });
    try kout.print("Implementation ID: {}\n", .{arch.mod.sbi.base.impl_id()});
    try kout.print("Implementation version: {}\n", .{arch.mod.sbi.base.impl_version()});
    try kout.print("Vendor ID: {}\n", .{arch.mod.sbi.base.vendor_id()});
    try kout.print("Arch ID: {}\n", .{arch.mod.sbi.base.arch_id()});
    try kout.print("Machine implementation ID: {}\n", .{arch.mod.sbi.base.machine_impl_id()});

    alloc.vmem.bootstrap();

    var vmem = alloc.vmem.Vmem{
        .name = "Main",
        .quantum = 4096,
    };
    try vmem.add(0, 0x1000);
    const allocation = try vmem.alloc(.{ .size = 0x10 }, .BestFit);

    try kout.print("New vmem arena {s} allocated {}", .{ vmem.name, allocation });

    while (true) {}
}

export fn kentry() noreturn {
    _ = kmain() catch |err| {
        var kernout = kstdout.Kstdout{};
        const kout = kernout.writer();

        kout.print("{}\n", .{err}) catch {};
    };

    while (true) {}
}
