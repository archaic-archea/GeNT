const std = @import("std");
pub const base = @import("./base.zig");
pub const time = @import("./time.zig");
pub const ipi = @import("./ipi.zig");

pub const SbiArgs = struct {
    arg0: usize = 0,
    arg1: usize = 0,
    arg2: usize = 0,
    arg3: usize = 0,
    arg4: usize = 0,
    arg5: usize = 0,
};

pub const SbiErr = error{
    None,
    Failed,
    NotSupported,
    InvalidParams,
    Denied,
    InvalidAddress,
    AlreadyAvailable,
    AlreadyStarted,
    AlreadyStopped,
    SharedMemTaken,
};

fn err_from_usize(val: isize) SbiErr {
    return switch (val) {
        0 => SbiErr.None,
        -1 => SbiErr.Failed,
        -2 => SbiErr.NotSupported,
        -3 => SbiErr.InvalidParams,
        -4 => SbiErr.Denied,
        -5 => SbiErr.InvalidAddress,
        -6 => SbiErr.AlreadyAvailable,
        -7 => SbiErr.AlreadyStarted,
        -8 => SbiErr.AlreadyStopped,
        -9 => SbiErr.SharedMemTaken,
        else => SbiErr.Failed,
    };
}

pub const SbiRet = struct {
    err: isize,
    val: usize,
};

pub fn sbi_call(eid: usize, fid: usize, args: SbiArgs) SbiErr!usize {
    const result = sbi_rawcall(args.arg0, args.arg0, args.arg0, args.arg0, args.arg0, args.arg0, fid, eid);

    if (result.err != 0) {
        return err_from_usize(result.err);
    }

    return result.val;
}

fn sbi_rawcall(arg0: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize, fid: usize, eid: usize) SbiRet {
    var err: isize = 0;
    var val: usize = 0;

    asm volatile (
        \\ecall
        : [err] "={a0}" (err),
          [val] "={a1}" (val),
        : [arg0] "{a0}" (arg0),
          [arg1] "{a1}" (arg1),
          [arg2] "{a2}" (arg2),
          [arg3] "{a3}" (arg3),
          [arg4] "{a4}" (arg4),
          [arg5] "{a5}" (arg5),
          [fid] "{a6}" (fid),
          [eid] "{a7}" (eid),
        : "a0", "a1"
    );

    return SbiRet{
        .err = err,
        .val = val,
    };
}
