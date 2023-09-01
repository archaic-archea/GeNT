# OPFS
Opfs (Process File System) is a file system designed to maintain a philosophy around everything being a process.  
The core features of this file system are:
* Journaling
* Custom attributes - Defined in the `attributes` section
* 64 bit file addressing
* 64 bit block addressing
* Page cache
* Metadata cache

## Layout
The first 32 bytes are reserved for the OPFS allocation range structure (defined just below).   
The next 16 bytes are reserved for a process ID (if a process)

```c
struct OPFSRange {
    /// Data section
    uint64_t dbase;
    uint64_t dlen;

    /// Journal section
    uint64_t jbase;
    uint64_t jlen;
}
```

## Block Allocation
Block allocation should be handled in 2 ranges.  
The data block, and the journal block.  
The journal block extends down from the top of a partition.

## Files/Directories
File/Directory names are expected to be 256 bytes where 1 byte represents 1 character.