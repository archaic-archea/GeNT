# Copyright
© 2023 Lilly & GeNT Developers archaic.archea@gmail.com  
License: [CNPL](https://git.pixie.town/thufie/npl-builder/raw/branch/main/cnpl.md)

# GeNT
This file covers the general architecture, and planned aspects of, GeNT

## Sections
GeNT is split into 3 types of sections, each section has specific use cases that allow for us to save on as much memory as possible.
The three sections are labeled the following
* Init          - Data/code only used during initialization, this memory will be released as soon as init mode is exitted
* Non-swappable  - Any data/code that is CRITICAL, and can NEVER be stored on disk while the OS is running, this code is guaranteed to always be in memory, sometimes for speed, sometimes because its required to access storage, as well as other reasons
* Swappable      - Data/Code that can be stored in non-volatile storage for later OS use, this includes some outer parts of the OS, as well as most tasks, although tasks can request that they are non-swappable
  
## Core Modules
GeNT is split up into several Core Modules, each of these serves a purpose in the OS.
* Init          - The Init core module is the same as the section specified in Sections, this is only used at the beginning, and is used for initialization. See the Init section of this document for more information
* Exit-Init     - Remove the Init components from memory
* Swapping      - A module dedicated to managing loading things into and out of RAM as needed to maintain free space, more information can be found in the Swap section.
* Hardware Abstraction Layer - The module responsible for providing an architecture-independent abstraction for different concepts more info in the section HAL

## Init
The initialization process consists of starting up every Core on the system, setting up storage drivers, interrupt set up, among other system specific initialization processes.

## Swap
In some systems memory may not be very available, so we can use swapping to free up space in RAM, this can be applied on some parts of the kernel, as well as the majority of tasks. First when swapping out a page, we look for an available spot in swap storage, and once we find a valid block, we copy the page of RAM into it, and the in the PTE associated with the page, we store the ID of the spot in swap.

## HAL
The hardware abstraction layer is made up of a few key points. First theres the paging abstraction, this prevents each and every coder from having to interact with the AS(Architecture specific) features about paging, such as page size, page entries, etc. Next is the interrupt abstraction, this covers up the AS interrupt systems, such as interrupt priority, whether or not MSIs are enabled, and actual transport methods on IO devices. Finally, the last abstraction point is the disk abstraction layer, this prevents coders from having to interact with the drivers for disks directly, as disk drivers are built into the kernel due to their importance in kernel operation.

## Drivers
Drivers are expected to most of the time be provided in user-space, but can be loaded in kernel-space. Examples of kernel-space drivers are file system drives, non-volatile storage drivers, as well as serial drivers.

## ISA Support
GeNT is intended as primarily a RISC-V OS, but support for other architectures, such as x64, or arm64 is planned eventually.  
If you would like to implement support for another architecture in the kernel please ensure it has access to a bootloader that implements the limine protocol.

## Protocol
The operating system relies on implementation of the Limine boot protocol, and can be booted on an architecture as long as support is implemented in the kernel, as well as it supporting limine.