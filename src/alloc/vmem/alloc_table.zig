const seg_list = @import("./seg_list.zig");
const vmem = @import("./vmem.zig");

const SegmentQueue = seg_list.SegmentQueue;
const Bt = vmem.Bt;
const BtKind = vmem.BtKind;
const NUM_HASH_BUCKETS = vmem.NUM_HASH_BUCKETS;

pub const AllocationTable = struct {
    len: usize = 0,
    buckets: [NUM_HASH_BUCKETS]SegmentQueue = [_]SegmentQueue{.{}} ** NUM_HASH_BUCKETS,

    pub fn is_empty(self: *const @This()) bool {
        return self.len == 0;
    }

    /// Returns a pointer to the bucket for the specified `base`.
    pub fn bucket_for_base(self: *const @This(), base: usize) *const SegmentQueue {
        const hash = murmur(base);
        return &self.buckets[hash & (NUM_HASH_BUCKETS - 1)];
    }

    /// Returns a mutable pointer to the bucket for the specified `base`.
    pub fn bucket_for_base_mut(self: *@This(), base: usize) *SegmentQueue {
        const hash = murmur(base);
        return &self.buckets[hash & (NUM_HASH_BUCKETS - 1)];
    }

    /// Insert a boundary tag into the allocation hash table
    pub fn insert(self: *@This(), bt: *Bt) void {
        self.bucket_for_base_mut(bt.base).insert_head(bt);
        self.len += 1;
    }

    /// Remove a boundary tag from the allocation hash table
    pub fn remove(self: *@This(), bt: *Bt) void {
        self.bucket_for_base_mut(bt.base).remove(bt);
        self.len -= 1;
    }

    /// Find a boundary tag in the hash table.
    ///
    /// If found, the boundary tag is *not* removed from the table.
    pub fn find(self: *const @This(), base: usize) ?*Bt {
        const bucket = self.bucket_for_base(base);
        var bt = bucket.head;

        while (bt != null) {
            if (bt.base == base and bt.kind != .Span) {
                break;
            }

            bt = bt.segment_queue_link.next;
        }

        return bt;
    }
};

const std = @import("std");
const builtin = @import("builtin");

fn murmur(first_key: usize) usize {
    var key = first_key;

    comptime if (builtin.current.ptrBitWidth() == 64) {
        key ^= key >> 33;
        key = key *% 0xff51afd7ed558ccd;
        key ^= key >> 33;
        key = key *% 0xc4ceb9fe1a85ec53;
        key ^= key >> 33;
    } else if (builtin.current.ptrBitWidth() == 32) {
        key ^= key >> 16;
        key = key *% 0x85ebca6b;
        key ^= key >> 13;
        key = key *% 0xc2b2ae35;
        key ^= key >> 16;
    } else {
        @compileError("Only supports 64 and 32 bit pointers");
    };

    return key;
}
