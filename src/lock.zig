const std = @import("std");

pub fn Mutex(comptime t: type) type {
    return struct {
        val: t,
        lock: std.atomic.Atomic(bool) = std.atomic.Atomic(bool).init(false),

        pub fn spinlock(self: *@This()) *t {
            while (self.lock.compareAndSwap(false, true, .AcqRel, .Acquire) == null) {
                std.atomic.spinLoopHint();
            }

            return &self.val;
        }

        pub fn unlock(self: *@This()) void {
            self.lock.store(false, .Release);
        }
    };
}
