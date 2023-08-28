# Events
Events (specified in `Userspace/events/events.md`) also must interact with the OS kernel, this ensures messages can actually reach their end point.
The kernel is expected to route an event's information to the associated process, and port. These events can be faults, OS messages, or IPC events.  

## Faults
Faults are handled by first entering the trap's fault, then information is loaded about what caused the fault, and its stored into a Fault Header (specified in `Userspace/events/faults.md`). Then a program must handle the fault itself, this can be by emulating an instruction, or by manually reading an unaligned value. The program then can read/write to the registers of the thread and write to the OS Com Port (specified in `Userspace/events/OS.md`) to signify changes were made that should fix the thread.