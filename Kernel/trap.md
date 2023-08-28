# Traps
This document lays out how traps are generally handled by the operating system, but this may vary from ISA to ISA.  
All traps should cause a switch to the core's interrupt stack, which should be allocated as 256 KiB at run-time, and the top address on the stack should be stored in the processors Information Block, which is a thread local variable

## Trap-shim Function
The trap shim function handles ISA specific trap entry and should always call the Trap-main function
All ISA specific trap entry details can be found under `isa/{isa}/trap.md`

## Trap-main Function
Arguments:
* Trap number - A number identifying what type of trap occured, trap types are specified in the Trap Type section
* Register frame - A copy of all the registers when the trap occurred

## Trap info
Trap information can be gathered through ISA-independent functions, as well as a structure containing information on the program that generated the trap.
The afforementioned ISA-independent functions, when called, will call ISA-dependent functions based off what the code was compiled for.  
Trap information functions should include the following:
* Page fault access address
* Invalid memory access address
* Unaligned access address
* Faulting instruction address
The trap information structure should include:
* Registers of the faulting program
* The program's page table

## Trap Types
A trap type can be first identified by the highest bit in the number, which indicates a external trap(value 1) or an internal trap(value 0)
### Internal Traps
* 0 - Unaligned instruction
* 1 - Unaligned load
* 2 - Unaligned store

* 8 - Execution of Invalid Memory
* 9 - Load of Invalid Memory
* 10 - Store of Invalid Memory

* 16 - Execution page fault
* 17 - Load page fault
* 18 - Store page fault

* 24 - Unknown instruction
* 25 - Breakpoint

* 33 - System call

### External Traps
* 0 - IPI
* 1 - Timer
* 2 - External device

## Handling Trap Types
Traps may have ISA specific methods of handling a trap, which should be handled with an ISA-specific function called `{trap_type}_handler`, details can be found in `isa/{isa}/trap.md`  
  
### From user space
* Unaligned access faults - If a handler is provided, pause the current thread and generate a new one on the handler function, otherwise shutdown the program
* Invalid memory access fault - Shutdown the user program
* Page fault - Check if its a fault on a swapped page, if so swap it back and continue, if that doesnt work then check if its in the virtual memory space for stacks and if it is, map more stack space, otherwise shutdown the user program
* Unknown instruction fault - If a handler is provided, pause the current thread and generate a new one on the handler function, otherwise shutdown the program
* System calls - Jump to the system call handler
* Breakpoint - Pause execution of the program, it can be unpaused later by a debugger or other program by using the `clear_breakpoint` function in the RDL

### From kernel space
* Unaligned access faults - Enter lockdown mode and load debugging info (as specified in `lockdown.md`)
* Invalid memory access fault - Enter lockdown mode and load debugging info (as specified in `lockdown.md`)
* Page fault - Check if its a fault on a swapped page, if so swap it back and continue, otherwise enter lockdown mode and load debugging info (as specified in `lockdown.md`)
* Unknown instruction fault - First check if the instruction is valid, if so attempt to emulate the instruction, if it can't, then enter lockdown mode and load debugging info (as specified in `lockdown.md`)
* IPI - View the IPI communication block, specified in the `IPI` section of this document
* Timer - drop the current timer list entry, complete any associated actions, and advance the timer list
* External device - Pass info to the device driver so it can handle it

## IPI
IPIs (Inter-Processor Interrupts) can be used to communicate with the other processors in a system, to smooth out this communication an IPI communication block is designed. This block takes the form of a queue of different messages for the interrupted core.  
When you would like to send information to another core you must first lock onto the processor's communication block, then you will push another entry onto it. These entries hold an IPI ID (Specified in `IPI IDs` in this document), as well as a pointer to a IPI-specific structure informing about this IPI that is occuring, and what might need done.

```c
struct ipi_queue_entry {
    /// IPI ID
    uint64_t id;
    void *data;
}
```

### IPI IDs
IPI IDs help identify why an IPI has occured, and how an interrupted core should respond.
* 0x00 => Invalid IPI, remove entry and continue execution
* 0x01 => Address space flush, flush the TLB, then continue execution
* 0x02 => Process port write, flush the TLB, read the data address as if it's a Kernel Event Signal (Specified in `Kernel/events/events.md`), spawn the handler, then resume execution