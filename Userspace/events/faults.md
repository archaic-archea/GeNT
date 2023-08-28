# Faults
Faults to userspace programs have their own message structures from IPC. These structures communicate information about faults occuring so a process can handle recovery. When a fault does occur, a process may want to make edits to its memory, so it can request over the OS Com Port (Specified in )

## Fault Header
The Fault Header is the primary structure for fault communication. It contains information on the fault ID, and where it occured.
```c
struct fault_header {
    /// Fault ID
    uint64_t id;
    /// Faulting address
    void *addr;
}
```