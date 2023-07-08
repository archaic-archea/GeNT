pub const ClaimErr = error{
    NoContiguous,
    NoEntry,
};

pub const FreeEntry = extern struct {
    prev: ?*Page,
    next: ?*Page,

    pub fn len(self: *FreeEntry) u64 {
        _ = self;
        return 4096;
    }

    pub fn grab_stitch(self: *FreeEntry) void {
        var prev = self.prev;
        var next = self.next;

        if (prev) |nonnull_prev| {
            nonnull_prev.entry.next = next;
        }

        if (next) |nonnull_next| {
            nonnull_next.entry.prev = prev;
        }
    }
};

pub const Page = extern union {
    entry: FreeEntry,
    data: [4096]u8,
};

pub const FreeList = struct {
    head: ?*Page,
    tail: ?*Page,
    len: u64,

    /// Init a new entry for the free list
    pub fn init(self: *FreeList, base: *void) void {
        self.head = @ptrCast(*Page, @alignCast(16, base));
        self.tail = @ptrCast(*Page, @alignCast(16, base));

        self.head.?.entry.prev = null;
        self.head.?.entry.next = null;

        self.len = 1;
    }

    /// Push a new entry onto the free list
    pub fn push(self: *FreeList, base: *void) void {
        var cur_tail = self.tail;

        self.tail = @ptrCast(*Page, @alignCast(16, base));
        cur_tail.?.entry.next = @ptrCast(*Page, @alignCast(16, base));

        self.tail.?.entry.prev = cur_tail;
        self.tail.?.entry.next = null;

        self.len += 1;
    }

    /// Pushes the new entry, makes sure it doesnt exist already, and also places it inbetween the logical next and previous entries
    pub fn push_org(self: *FreeList, base: *void) void {
        var base_ptr: *Page = @ptrCast(*Page, @alignCast(16, base));
        var iter = self.iterf();
        var entry = iter.next();

        var rep_null = false;

        while (@as(usize, base) > @as(usize, entry.?)) {
            if (rep_null == true) {
                @panic("NO INSERTION POINT FOUND");
            }

            entry = iter.next();
            if (entry == null) {
                rep_null = true;
            }
        }

        if (@as(usize, base) == @as(usize, entry.?)) {
            @panic("Both the base and entry ptrs are the same");
        }

        var next_page = entry;
        var prev_page = entry.?.entry.prev;

        var next_entry = &next_page.?.entry;
        var prev_entry = &prev_page.?.entry;

        base_ptr.entry.next = next_page;
        base_ptr.entry.prev = prev_page;

        next_entry.prev = base_ptr;
        prev_entry.next = base_ptr;
    }

    pub fn push_sec_org(self: *FreeList, base: *void, frames: u64) void {
        var base_int = @as(usize, base);

        var frame = 0;
        while (frame < frames) {
            var addr = @as(*void, base_int + frame * 4096);

            self.push_org(addr);

            frame += 1;
        }
    }

    /// Index forward, from the head
    pub fn indexf(self: *FreeList, idx: u64) ?*Page {
        // Check that the index isnt more than the length
        if (idx >= self.len) {
            return null;
        }

        var current_ptr = self.head;

        var loop_index = 0;
        while (loop_index < idx) {
            current_ptr = current_ptr.next;
            loop_index += 1;
        }

        return current_ptr;
    }

    /// Index backward, from the tail
    pub fn indexb(self: *FreeList, idx: u64) ?*Page {
        // Check that the index isnt more than the length
        if (idx >= self.len) {
            return null;
        }

        var current_ptr = self.tail;

        var loop_index = idx;
        while (loop_index < self.len) {
            current_ptr = current_ptr.prev;

            loop_index += 1;
        }

        return current_ptr;
    }

    /// Provide a forward iterator over the entries, any changes to the list while iterating will only affect un-read entries
    pub fn iterf(self: *FreeList) FreeIter {
        return .{ .current = self.head };
    }

    /// Provide a backward iterator over the entries, any changes to the list while iterating will only affect un-read entries
    pub fn iterb(self: *FreeList) FreeIter {
        return .{ .forward = false, .current = self.tail };
    }

    pub fn iter_at(self: *FreeList, idx: u64, forward: bool) FreeIter {
        return .{ .forward = forward, .current = self.indexf(idx).? };
    }

    /// Claim a single frame of memory
    pub fn claim(self: *FreeList) ClaimErr!*void {
        var iter = self.iterf();

        while (iter.next()) |entry| {
            var prev = entry.entry.prev;
            var next = entry.entry.next;

            if (prev) |nonnull_prev| {
                if (@as(usize, nonnull_prev) > 0x80000000) {
                    nonnull_prev.entry.next = next;
                }
            }

            if (next) |nonnull_next| {
                if (@as(usize, nonnull_next) > 0x80000000) {
                    nonnull_next.entry.prev = prev;
                }
            }

            var ret: *void = @ptrCast(*Page, @alignCast(16, entry));

            return ret;
        }

        return ClaimErr.NoEntry;
    }

    pub fn grab_stitch(self: *FreeList, idx: u64) ?*void {
        if (self.indexf(idx)) |entry| {
            var prev = entry.prev;
            var next = entry.next;

            if (prev) |nonnull_prev| {
                nonnull_prev.next = next;
            }

            if (next) |nonnull_next| {
                nonnull_next.prev = prev;
            }

            return @ptrCast(*void, entry);
        }

        return null;
    }

    /// Claim a contiguous section of memory of a specific frame length
    pub fn contiguous_claim(self: *FreeList, frames: u64) ClaimErr!*void {
        var iter = self.iterf();

        var cur_base = iter.next();

        var prev_base = cur_base;
        var length: u64 = 1;

        while (iter.next()) |entry| {
            if (@as(usize, prev_base.?) + 4096 == @as(usize, entry)) {
                length += 1;
            } else {
                length = 1;
                cur_base = entry;
            }

            if (length == frames) {
                var claim_iter: FreeIter = .{
                    .forward = true,
                    .current = cur_base,
                };

                var loop_index = 0;
                while (loop_index < length) {
                    var claim_entry_opt = claim_iter.next();
                    if (claim_entry_opt) |claim_entry| {
                        claim_entry.entry.grab_stitch();
                    }

                    loop_index += 1;
                }

                return @ptrCast(*void, cur_base);
            }

            prev_base = entry;
        }

        return ClaimErr.NoContiguous;
    }
};

pub const FreeIter = struct {
    forward: bool = true,
    current: ?*Page = null,

    /// Returns the current entry while setting self.current to the next entry
    pub fn next(self: *FreeIter) ?*Page {
        var ret = self.current;

        if (ret == null) {
            return null;
        }

        self.current = switch (self.forward) {
            true => self.current.?.entry.next,
            false => self.current.?.entry.prev,
        };

        return ret;
    }

    pub fn prev(self: *FreeIter) ?*Page {
        var ret = self.current;

        if (ret == null) {
            return null;
        }

        self.current = switch (self.forward) {
            true => self.current.?.entry.prev,
            false => self.current.?.entry.next,
        };

        return ret;
    }
};

const std = @import("std");
const Atomic = std.atomic.Atomic;
const Ordering = std.atomic.Ordering;

pub var FREE_MEM = PmmMutex{
    .data = FreeList{
        .head = null,
        .tail = null,
        .len = 0,
    },
    .claimed = Atomic(bool).init(false),
};

pub const PmmMutex = struct {
    data: FreeList,
    claimed: Atomic(bool),

    pub fn lock(self: *PmmMutex) PmmMutexGuard {
        while (self.claimed) {}
        self.claimed.store(true, Ordering.Unordered);
    }

    pub fn lockOrPanic(self: *PmmMutex) PmmMutexGuard {
        if (self.claimed.load(Ordering.Unordered)) {
            @panic("PMM ALREADY LOCKED");
        }

        self.claimed.store(true, Ordering.Unordered);

        return PmmMutexGuard{
            .mutex = self,
        };
    }
};

pub const PmmMutexGuard = struct {
    mutex: *PmmMutex,

    pub fn unlock(self: *PmmMutexGuard) void {
        self.mutex.claimed.store(false, Ordering.Unordered);
    }

    pub fn deref(self: *PmmMutexGuard) *FreeList {
        return &self.mutex.*.data;
    }
};
