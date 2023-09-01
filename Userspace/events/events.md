# Events
In GeNT if a process wishes, it may set different handler functions for 'events' these events are triggered by IO, some faults, and IPC.
Events have different handlers with a defined ABI, information on the ABI can be found in the `ABI` section of this document.

## ABI
When an event occurs the operating system will spin up a new thread (or use a cached thread, more information in `Userspace/threads/cached.md`).
On start up, this new thread has an initialized TLS, new stack, and properly set registers.   
Arguments will be passed on the stack in the form of a pointer to an event-defined structure.
Events are listed in the `Event Port List` section

## Event Ports List
0x00 => IPC init port, see `Userspace/IPC/Ports.md` for more info
0x01 => Fault event port, see the `Faults` section in this document
0x02 => OS Com port, see `Userspace/IPC/OS.md` for more information
0x03 => Init Com port, see `Userspace/io.md` for more information
0x04..0xFFFFFFFF => Proccess Com ports

## Faults
When a program faults and the OS cant handle it, it passes it to the program as a last ditch effort to fix whatever issue has occured.
These are provided to the program in the form of fault events, the program can handle this however it wants, and at the end it should provide the OS with a result as to whether or not it successfully handled the issue.
If the process faults within the handler, the process is immediately paused and the init process is expected to provide a program to handle it, if the init process cannot handle it, the process should be killed.
0x00 => Invalid Exception, ignore
0x01 => Invalid Instruction
0x02 => Unaligned Access