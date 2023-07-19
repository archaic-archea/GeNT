# Syscalls
The system call interface is not intended to be stable, but instead its intended for you to access it through a dynamic library that provides the basic functions required to interact with the kernel, and other applications.

## RDL
The Root Dynamic Library serves as a dynamically loaded library provided with every program that provides basic functions to interact with the operating system.  
A version of the RDL should be provided with the kernel in order to allow boot-strapping systems.  
the RDL can be changed in order to improve efficiency, clean up the interface, as well as for many other reasons, but different versions of the RDL must be available for every kernel version, although the one shipped with the kernel is expected to be the most recent version as of the day the kernel is compiled.  
Programs are expected to provide a stub that specifies their expected RDL version if they are native to the operating system.

## DL Chaining
Dynamic libraries can be chained together to act as compatability layers, this can help to run binaries for other operating systems, as long as they are on a matching architecture.