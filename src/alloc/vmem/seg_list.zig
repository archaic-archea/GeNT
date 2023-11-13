const vmem = @import("./vmem.zig");
const std = @import("std");
const builtin = @import("builtin");

const Bt = vmem.Bt;

pub const LinkedListLink = struct {
    prev: ?*Bt = null,
    next: ?*Bt = null,
};

/// Segment list
///
/// Manages a collection of [`Bt`]s using `segment_list_link`.
pub const SegmentList = struct {
    head: ?*Bt = null,
    tail: ?*Bt = null,

    fn insertion_point(self: *const @This(), base: usize) .{ ?*Bt, ?*Bt } {
        var prev = null;
        var next = self.head;

        while (next != null) {
            if (next.base > base) {
                break;
            }

            prev = next;
            next = next.segment_list_link.next;
        }

        return .{ prev, next };
    }

    pub fn insert_ordered(self: *@This(), bt: *Bt) void {
        const vals = self.insertion_point(bt.base);

        const prev = vals[0];
        const next = vals[1];

        if (prev == null) {
            self.head = bt;
        } else {
            prev.segment_list_link.next = bt;
        }

        if (next == null) {
            self.tail = bt;
        } else {
            next.segment_list_link.prev = bt;
        }
    }

    pub fn insert_ordered_span(self: *@This(), span: *Bt, free: *Bt) void {
        comptime if (builtin.current.mode == std.builtin.Mode.Debug) {
            if (span.base != free.base) {
                @panic("Free base was not equal to the span base");
            }

            if (span.size != free.size) {
                @panic("Free size was not equal to the span size");
            }
        };

        const vals = self.insertion_point(span.base);

        const prev = vals[0];
        const next = vals[1];

        span.segment_list_link.prev = prev;
        span.segment_list_link.next = free;
        free.segment_list_link.prev = span;
        free.segment_list_link.next = next;

        if (prev == null) {
            self.head = span;
        } else {
            prev.segment_list_link.next = span;
        }

        if (next == null) {
            self.tail = free;
        } else {
            next.segment_list_link.prev = free;
        }
    }

    pub fn remove(self: *@This(), bt: *Bt) void {
        const prev = bt.segment_list_link.prev;
        const next = bt.segment_list_link.next;

        if (prev == null) {
            self.head = next;
        } else {
            prev.segment_list_link.next = next;
        }

        if (next == null) {
            self.tail = prev;
        } else {
            next.segment_list_link.prev = prev;
        }
    }
};

/// Segment queue
///
/// Manages a collection of [`Bt`]s using `segment_queue_link`.
pub const SegmentQueue = struct {
    head: ?*Bt = null,
    tail: ?*Bt = null,

    pub fn insert_head(self: *@This(), bt: *Bt) void {
        const prev = null;
        const next = self.head;

        bt.segment_queue_link.prev = prev;
        bt.segment_queue_link.next = next;

        self.head = bt;

        if (next == null) {
            self.tail = bt;
        } else {
            next.?.segment_queue_link.prev = bt;
        }
    }

    pub fn remove(self: *@This(), bt: *Bt) void {
        const prev = bt.segment_queue_link.prev;
        const next = bt.segment_queue_link.next;

        if (prev == null) {
            self.head = next;
        } else {
            prev.segment_queue_link.next = next;
        }

        if (next == null) {
            self.tail = prev;
        } else {
            next.segment_queue_link.prev = prev;
        }
    }
};
