const std = @import("std");
const stdio = @import("io/writer/stdio.zig");
const stderr = @import("io/writer/stderr.zig");
const fb = @import("io/framebuffer/framebuffer.zig");

pub fn getStdWriter(framebuffer: fb.FrameBuffer) stdio.Kstdout {
    var stdiowriter = stdio.Kstdout{ .framebufferwriter = framebuffer };

    return stdiowriter;
}

pub fn getStdErr() stderr.Kstderr {
    return .{};
}