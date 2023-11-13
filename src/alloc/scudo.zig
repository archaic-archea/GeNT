// An implementation of a SCUDO hardened allocator as specified [here](https://trenchant.io/scudo-hardened-allocator-unofficial-internals-documentation/)

// Here are the options used in the allocator

/// Size (in kilobytes) of quarantine used to delay the actual deallocation of chunks.
/// Lower value may reduce memory usage but decrease the effectiveness of the mitigation
const quarantine_size: usize = 0;

/// Size (in kilobytes) of per-hardware-thread cache used to offload the global quarantine.
/// Lower value may reduce memory usage but might increase the contention on the global quarantine
const tls_quarantine_size: usize = 0;

/// Size (in bytes) up to which chunks will be quarantined (if lower than or equal to)
const quantine_max_chunk_size: usize = 0;

/// Terminate on a type mismatch in allocation-deallocation functions
/// eg: malloc/delete, new/free, new/delete[], etc
const dealloc_type_mismatch: bool = false;

/// Terminate on a size mismatch between a sized-delete and the actual size of a chunk (as provided to new/new[])
const delete_size_mismatch: bool = true;

/// Zero chunk contents on allocation
const zero_contents: bool = false;

/// Pattern fill chunk contents on allocation
const pattern_contents: bool = false;

/// Indicate whether the allocator should terminate instead of returning NULL in otherwise non-fatal error scenarios
/// eg: OOM, invalid allocation alignments, etc
const may_null: bool = true;

/// Entries to keep in the allocation ring buffer for SCUDO
const alloc_ring_buf: usize = 32768;
