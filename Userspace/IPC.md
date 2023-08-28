# IPC
IPC (Inter-Proccess Communication) is handled with events, when initializing an IPC channel a program should write to the standardized IPC init port (specified in `Userspace/events/events.md`, in section `Event Ports List`) with data about a port to write back to and the reason for the request in the form of an IPC Header (specified in `Structures`), then the program receiving the initialization request should respond with information on whether it accepts the channel request once again in the form of an IPC Header. If it doesnt accept it, then it returns no port, and no buffer, if it does accept it, it should allocate a port for that program, load a handler, and send back the new port, and any information about the connection. All interactions on that channel should be on the new port you provide.

## Structures
```c
struct ipc_header {
    /// Process ID to talk to after
    size_t proc_id;
    /// Port to write to in response
    uint64_t resp_port;

    /// Buffer containing connection info
    /// The kernel should convert this to a pointer to shared memory for the receiving program
    uint8_t *buf;
    size_t len;
}
```