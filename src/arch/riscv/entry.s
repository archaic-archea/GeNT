.section .text
.option norvc
 
.type start, @function
.global start
start:
	.cfi_startproc

    lla sp, __stack_top
    li tp, 0x0
    lla gp, __global_pointer

	lla t0, hlt_loop
	csrw stvec, t0
 
	j kentry

	j hlt_loop
 
	.cfi_endproc

hlt_loop:
	wfi
	j hlt_loop