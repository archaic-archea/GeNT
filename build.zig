const std = @import("std");

// Although this function looks imperative, note that its job is to
// declaratively construct a build graph that will be executed by an external
// runner.
pub fn build(b: *std.Build) !void {
    var target = try std.zig.CrossTarget.parse(.{
        .arch_os_abi = "riscv64-freestanding",
        .cpu_features = "baseline+m+a+c",
    });

    // Standard optimization options allow the person running `zig build` to select
    // between Debug, ReleaseSafe, ReleaseFast, and ReleaseSmall. Here we do not
    // set a preferred release mode, allowing the user to decide how to optimize.
    const optimize = b.standardOptimizeOption(.{});

    const kernel = b.addExecutable(.{
        .name = "GeNT",
        .root_source_file = .{ .path = "src/main.zig" },
        .single_threaded = false,
        .target = target,
        .optimize = optimize,
        .use_llvm = true,
    });

    kernel.setLinkerScript(.{ .path = "./linker.lds" });
    kernel.addAssemblyFile(.{ .path = "./src/arch/riscv/entry.s" });
    kernel.code_model = .medium;
    kernel.rdynamic = true;
    kernel.pie = true;

    b.installArtifact(kernel);
}
