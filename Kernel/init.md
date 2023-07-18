# Initialization
First we enter through an entry function written in assembly, this should load a gp(if applicable), and then immediately jump to the main initialization function.

## Init-main
The init-main function should complete the following:
* Initialize the PMM
* Initialize other cores
* Initialize kernel level VMM
* Initialize non-volatile storage drivers
* Initialize partition manager
* Initialize swap manager
* Load and initialize all kernel level drivers provided
* Begin executive