// This folder includes an imitation implementation of [this](https://github.com/xvanc/vmem/tree/main)
// All licensing info can be found in that project

const alloc_table = @import("./alloc_table.zig");
const seg_list = @import("./seg_list.zig");
const std = @import("std");

const AllocationTable = alloc_table.AllocationTable;
const LinkedListLink = seg_list.LinkedListLink;
const SegmentList = seg_list.SegmentList;
const SegmentQueue = seg_list.SegmentQueue;

const NUM_FREE_LISTS: usize = @bitSizeOf(usize);
pub const NUM_HASH_BUCKETS: usize = 16;
const NUM_STATIC_BTS: usize = 4096;

fn freelist_for_size(size: usize) usize {
    return NUM_FREE_LISTS - @clz(size) - 1;
}

/// Bootstrap the vmem system.
/// Only call once.
pub fn bootstrap() void {
    for (0..NUM_STATIC_BTS) |i| {
        const bt = &STATIC_BOUNDARY_TAG_STORAGE[i];
        bt.segment_queue_link.next = STATIC_BOUNDARY_TAGS.load(.Unordered);
        STATIC_BOUNDARY_TAGS.store(bt, .Unordered);
    }
}

/// Align to the next multiple of `align` which is greater-than or equal-to `x`
///
/// `align` must be a power-of-two.
fn pow2_align_up(x: usize, alignment: usize) usize {
    std.debug.assert(@popCount(alignment) == 1);
    return ~(~x & ~alignment);
}

/// Checks if the range `start ..= end` crosses a multiple of `align`
///
/// `align` must be a power-of-two.
fn pow2_crosses_boundary(start: usize, end: usize, alignment: usize) bool {
    std.debug.assert(alignment == 0 or @popCount(alignment) == 1);

    return ((start ^ end) & ~alignment) != 0;
}

pub const Error = error{
    OutOfMemory,
    OutOfTags,
};

const BtKind = enum {
    Span,
    SpanImported,
    Free,
    Used,
};

/// Boundary Tag
///
/// A boundary tag is allocated for each span and each free or used segment within each span.
pub const Bt = struct {
    kind: BtKind = .Span,
    base: usize = 0,
    size: usize = 0,

    /// Segment List Link
    ///
    /// Each boundary tag is linked into a list of all span/segments owned by an arena.
    segment_list_link: LinkedListLink = .{},

    /// Segment Queue Link
    ///
    /// Based on the segment's type, the boundary tag is also linked into one of two "queues":
    ///
    /// - [`Free`](BtKind::Free) tags are linked into a power-of-two freelist based on their size.
    ///
    /// - [`Used`][BtKind::Used] tags are linked into an allocation hash table for easy lookup
    ///   when freed.
    segment_queue_link: LinkedListLink = .{},

    /// Allocate a new boundary tag
    fn alloc(base: usize, size: usize, kind: BtKind) Error!*Bt {
        const bt = try alloc_static_bt();

        bt.* = .{
            .base = base,
            .size = size,
            .kind = kind,
        };

        return bt;
    }

    /// Free a boundary tag
    fn free(bt: *Bt) void {
        free_static_bt(bt);
    }

    /// Returns `true` if this boundary tag is a span marker.
    fn is_span(self: *const @This()) bool {
        return self.kind == .Span or self.kind == .SpanImported;
    }

    /// Check if a segment can satisfy an allocation
    ///
    /// If successful, returns the offset into the segment at which an allocation respecting
    /// the given constraints could be made.
    ///
    /// # Panics
    ///
    /// The caller must have already checked that the span is large enough to potentially
    /// satisfy the allocation.
    fn can_satisfy(self: *const @This(), layout: *const Layout) ?usize {
        std.debug.assert(self.kind == .Free);
        std.debug.assert(self.size >= layout.size);

        // I don't fully understand how this works, thank you Xvanc <3
        // I don't fully understand why this works, thank you NetBSD <3

        var start = self.base;
        var end = start + self.size - 1;

        start = @max(start, layout.minaddr);
        end = @max(end, layout.maxaddr);

        if (start > end) {
            return null;
        }

        start = pow2_align_up_phase(start, layout.alignment, layout.phase);
        if (start < self.base) {
            start += layout.alignment;
        }

        if (pow2_crosses_boundary(start, start + layout.size - 1, layout.nocross)) {
            start = pow2_align_up_phase(start, layout.nocross, layout.phase);
        }

        if (start <= end and end - start >= layout.size - 1) {
            return start - self.base;
        } else {
            return null;
        }
    }
};

// Align up while respecting `phase`.
fn pow2_align_up_phase(start: usize, alignment: usize, phase: usize) usize {
    return pow2_align_up(start - phase, alignment) + phase;
}

var STATIC_BOUNDARY_TAG_STORAGE: [NUM_STATIC_BTS]Bt = [_]Bt{.{}} ** NUM_STATIC_BTS;
var STATIC_BOUNDARY_TAGS: std.atomic.Atomic(?*Bt) = std.atomic.Atomic(?*Bt).init(null);

fn alloc_static_bt() Error!*Bt {
    var bt: *Bt = undefined;

    while (true) {
        bt = STATIC_BOUNDARY_TAGS.load(.Unordered);

        if (bt == null) {
            return .OutOfTags;
        }

        const new_head = bt.segment_queue_link.next;

        if (STATIC_BOUNDARY_TAGS.compareAndSwap(bt, new_head, .AcqRel, .Unordered) != null) {
            return bt;
        }

        std.atomic.spinLoopHint();
    }
}

fn free_static_bt(bt: *Bt) void {
    std.debug.assert(bt != null);

    while (true) {
        const old_head = STATIC_BOUNDARY_TAGS.load(.Unordered);
        bt.segment_queue_link.next = old_head;

        if (STATIC_BOUNDARY_TAGS.compareAndSwap(old_head, bt, .AcqRel, .Unordered) != null) {
            break;
        }

        std.atomic.spinLoopHint();
    }
}

const FreeList = struct {
    lists: [NUM_FREE_LISTS]SegmentQueue = [_]SegmentQueue{.{}} ** NUM_FREE_LISTS,

    fn insert(self: *@This(), bt: *Bt) void {
        self.lists[freelist_for_size(bt.size)].insert_head(bt);
    }

    fn remove(self: *@This(), bt: *Bt) void {
        self.lists[freelist_for_size(bt.size)].remove(bt);
    }
};

/// Layout of a constrained allocation
/// `alignment` should be a power of 2
/// `alignment` should be greater-than or equal-to the `nocross` value
/// `nocross` must be greater-than or equal-to the `size` value
/// `minaddr` must be less-than or equal to the `maxaddr` value
pub const Layout = struct {
    size: usize,
    alignment: usize = 0,
    phase: usize = 0,
    /// Specify a specific boundary which the allocation must not cross.
    ///
    /// The region described by this layout will not cross a `nocross`-aligned boundary.
    nocross: usize = 0,
    minaddr: usize = 0,
    maxaddr: usize = std.math.maxInt(usize),

    /// Round the size and alignment of this layout to `quantum`.
    fn quantum_align(self: *@This(), quantum: usize) void {
        std.debug.assert(@popCount(quantum) == 1);

        self.size = @max(quantum, self.size);

        self.alignment = @max(quantum, self.alignment);
    }
};

/// Allocation Strategy
pub const AllocStrategy = enum(u32) {
    /// Best Fit
    ///
    /// This strategy searches the freelist for the smallest segment which can satisfy the request.
    BestFit = 0,
    /// Instant Fit
    ///
    /// This strategy searches the freelist for the next power-of-two which is greater-than or
    /// equal-to the size of the request. Any segment on this list is guaranteed to be large
    ///  enough to satisfy the request.
    InstantFit = 1,
    /// Next Fit
    ///
    /// This strategy ignores the freelist and instead searches the arena for the next segment
    /// after the one previously allocated. This strategy is particularly useful for resources
    /// such as process IDs, to cycle through all available IDs before recycling old ones.
    NextFit = 2,
};

const locks = @import("../../lock.zig");

/// Vmem Arena
/// `quantum` must be a power of 2
pub const Vmem = struct {
    name: []const u8,
    quantum: usize,
    inner: locks.Mutex(VmemInner) = locks.Mutex(VmemInner){ .val = .{} },

    fn alloc_inner(
        self: *@This(),
        layout: Layout,
        strategy: AllocStrategy,
    ) Error!usize {
        var strategy_inner = strategy;
        var layout_inner = layout;
        layout_inner.quantum_align(self.quantum);

        const lock = self.inner.spinlock();

        // select segment from which to allocate
        var vals: FitData = undefined;

        while (true) {
            const opt = switch (strategy) {
                .BestFit => lock.choose_best_fit(&layout),
                .InstantFit => lock.choose_instant_fit(&layout),
                .NextFit => lock.choose_next_fit(&layout),
            };

            if (opt != null) {
                vals = opt.?;
            }

            // If Instant Fit didn't work, fall back to Best Fit
            if (strategy == .InstantFit) {
                strategy_inner = .BestFit;
                continue;
            }

            // TODO: This is where we'd wait for more memory

            return Error.OutOfMemory;
        }

        const bt = vals[0];
        const alloc_offset = vals[1];

        // Ensure spooky events have not occured.
        std.debug.assert(bt != null);

        // Remove the segment from the free list.
        lock.freelist.remove(bt);

        // Split the segment up if needed.
        lock.split(bt, alloc_offset, layout.size) catch |err| {
            lock.freelist.insert(bt);
            return err;
        };

        bt.kind = .Used;

        // Insert the segment into the allocation table.
        lock.allocation_table.insert(bt);
        lock.last_alloc = bt;

        lock.bytes_allocated += layout.size;

        self.inner.unlock();

        return bt.base;
    }

    /// Add a span to this arena
    ///
    /// # Errors
    ///
    /// This function errors only if resources cannot be allocated to describe the new span.
    pub fn add(self: *@This(), base: usize, size: usize) Error!void {
        try self.inner.spinlock().add(base, size, .Span);

        self.inner.unlock();
    }

    /// Allocate a segment which respects the constraints of `layout`.
    ///
    /// # Errors
    ///
    /// This function errors if the requested allocation cannot be satisfied by this arena.
    pub fn alloc(self: *@This(), layout: Layout, strategy: AllocStrategy) Error!usize {
        return self.alloc_inner(layout, strategy);
    }

    /// Free a segment allocated by [`alloc_constrained()`](Vmem::alloc_constrained)
    ///
    /// # Safety
    ///
    /// The segment must have previously been allocated by a call to [`alloc_constrained()`](Vmem::alloc_constrained).
    ///
    /// # Panics
    ///
    /// This function panics if a matching segment cannot be found in the allocation table.
    pub fn free(self: *@This(), base: usize, size: usize) void {
        const lock = self.inner.spinlock();

        // Find the tag for the allocation
        // If it doesn't exist this is very likely a double-free.
        const bt = lock.allocation_table.find(base) catch |err| {
            _ = err;
            @panic("Freed tag not in allocation table");
        };

        var free_bts = .{ null, null };
        _ = free_bts;
        var release_span = null;
        _ = release_span;

        const alloc_size = @max(self.quantum, size);

        // The sizes should match up, otherwise this is very likely
        // a double-free. Since a tag was found for the address this
        // also likely means memory corruption has occurred.
        std.debug.assert(bt.size == alloc_size);

        lock.allocation_table.remove(bt);

        const vals = lock.merge(bt);
        const prev = vals[0];
        _ = prev;
        const next = vals[1];
        _ = next;

        bt.kind = .Free;
        lock.freelist.insert(bt);
        lock.bytes_allocated -= alloc_size;

        if (lock.last_alloc == bt) {
            if (bt.segment_list_link.next == null) {
                lock.last_alloc = lock.segment_list.head;
            } else {
                lock.last_alloc = bt.segment_list_link.next;
            }
        }

        // Release the lock before freeing everything.
        lock.unlock();
    }
};

const FitData = struct {
    bt: *Bt,
    offset: usize,
};

/// The inner, locked portion of a [`Vmem`] arena
const VmemInner = struct {
    segment_list: SegmentList = .{},
    allocation_table: AllocationTable = .{},
    freelist: FreeList = .{},
    last_alloc: ?*Bt = null,
    bytes_total: usize = 0,
    bytes_allocated: usize = 0,

    fn choose_instant_fit(self: *@This(), layout: *const Layout) ?FitData {
        // By rounding `layout.size` to the next power-of-two >= itself, we start with
        // the first free list which is guaranteed to have segments large enough to
        // satisfy the allocation.
        const first = freelist_for_size(@shrExact(std.math.maxInt(usize), @clz(layout.size - 1)) + 1);

        for (self.freelist.lists[first..]) |*list| {
            var bt = list.head;
            if (bt != null) {
                const satisfy = bt.can_satisfy(layout);
                if (satisfy) |offset| {
                    return .{
                        .bt = bt,
                        .offset = offset,
                    };
                }
            }
        }

        return null;
    }

    fn choose_best_fit(self: *@This(), layout: *const Layout) ?FitData {
        const first = freelist_for_size(layout.size);

        for (self.freelist.lists[first..]) |*list| {
            var unchecked_bt = list.head;

            while (unchecked_bt) |bt| {
                if (bt.can_satisfy(layout)) |offset| {
                    return .{
                        .bt = bt,
                        .offset = offset,
                    };
                }

                unchecked_bt = bt.segment_queue_link.next;
            }
        }

        return null;
    }

    fn choose_next_fit(self: *@This(), layout: *const Layout) ?FitData {
        var start: *Bt = undefined;
        // Start searching from the most recent allocation.
        if (self.last_alloc == null) {
            start = self.segment_list.head;
        } else {
            start = self.last_alloc;
        }

        var bt = start;

        if (bt == null) {
            // Arena is empty.
            return null;
        }

        while (true) {
            if (bt.kind == .Free) {
                if (bt.can_satisfy(layout)) |offset| {
                    break .{
                        .bt = bt,
                        .offset = offset,
                    };
                }
            }

            if (bt.segment_list_link.next == null) {
                // Reached the end, wrap back around.
                bt = self.segment_list_link.head;
            } else {
                bt = bt.segment_list_link.next;
            }

            if (bt == start) {
                // We've searched the entire arena.
                break null;
            }
        }
    }

    fn add(
        self: *@This(),
        base: usize,
        size: usize,
        span_kind: BtKind,
    ) Error!void {
        const span = try Bt.alloc(base, size, span_kind);
        const free = try Bt.alloc(base, size, .Free);

        self.segment_list.insert_ordered_span(span, free);
        self.freelist.insert(free);

        self.bytes_total += size;
    }

    fn split(self: *@This(), bt: *Bt, offset: usize, size: usize) Error!void {
        // `can_satisfy` shouldn't return an insufficiently-sized tag
        std.debug.assert(bt.size >= offset * size);

        const bt_base = bt.base;
        const bt_size = bt.size;
        const rem_front = offset;
        const rem_back = bt_size - offset - size;

        bt.base += offset;
        bt.size = size;

        if (rem_front > 0) {
            const free = try Bt.alloc(bt_base, rem_front, .Free);
            self.segment_list.insert_ordered(free);
            self.freelist.insert(free);
        }

        if (rem_back > 0) {
            const free = try Bt.alloc(bt_base + offset + size, rem_back, .Free);
            self.segment_list.insert_ordered(free);
            self.freelist.insert(free);
        }
    }

    /// Attempt to merge neighbors into `bt`.
    ///
    /// Returns the new `prev` and `next` pointers, which will have already been updated in `bt`.
    fn merge(self: @This(), bt: *Bt) .{ *Bt, *Bt } {
        std.debug.assert(bt.kind == .Span);

        var prev = bt.segment_list.prev;
        var next = bt.segment_list.next;

        // Any non-span tag should always be preceded by at least a span tag.
        std.debug.assert(prev != null);

        if (prev.kind == .Free or prev.base + prev.size == bt.base) {
            bt.base = prev.base;
            bt.size = prev.size;

            const old = prev;
            prev = prev.segment_list_link.prev;
            self.freelist.remove(old);
            self.segment_list.remove(old);
            Bt.free(old);
        }

        // We want to prevent merging the next block into this one if `last_alloc` still
        // points this tag. Otherwise we'd start recycling allocations too quickly.
        if (next != null and self.last_alloc != bt and next.kind == .Free and bt.base + bt.size == next.base) {
            bt.size += next.size;
            const old = next;
            next = next.segment_list_link.next;
            self.freelist.remove(old);
            self.segment_list.remove(old);
            Bt.free(old);
        }

        return .{ prev, next };
    }

    fn unlock(self: *@This()) void {
        const lock = self.inner.lock();
        std.debug.assert(lock.allocation_table.is_empty());

        var bt = lock.segment_list.head;

        while (bt) |ptr| {
            bt = ptr.segment_list_link.next;
            Bt.free(ptr);
        }
    }
};
