# Trap 
TODO

## Trap Shim
```as
trap_shim:
    // Interrupts are disabled when we enter a trap

    // Swap t6 and sscratch, t6 will be switched back out later to 
    csrrw t6, sscratch, t6

    // Store current stack pointer temporarily
    sd sp, 24(t6)

    // Load kernel's stack pointer
    ld sp, 0(t6)

    addi sp, sp, {TRAP_FRAME_SIZE}

    // Store the userspace stack information
    sd ra, 8(sp)
    // Load and save the userspace stack pointer using
    // the now freed `ra` register
    ld ra, 24(t6)
    sd ra, 16(sp)
    // Save the other registers regularly
    sd gp, 24(sp)
    sd tp, 32(sp)
    sd t0, 40(sp)
    sd t1, 48(sp)
    sd t2, 56(sp)
    sd s0, 64(sp)
    sd s1, 72(sp)
    sd a0, 80(sp)
    sd a1, 88(sp)
    sd a2, 96(sp)
    sd a3, 104(sp)
    sd a4, 112(sp)
    sd a5, 120(sp)
    sd a6, 128(sp)
    sd a7, 136(sp)
    sd s2, 144(sp)
    sd s3, 152(sp)
    sd s4, 160(sp)
    sd s5, 168(sp)
    sd s6, 176(sp)
    sd s7, 184(sp)
    sd s8, 192(sp)
    sd s9, 200(sp)
    sd s10, 208(sp)
    sd s11, 216(sp)
    sd t3, 224(sp)
    sd t4, 232(sp)
    sd t5, 240(sp)
    ld tp, 8(t6)
    ld gp, 16(t6)
    // Swap `t6` and `sscratch` again
    csrrw t6, sscratch, t6
    sd t6, 248(sp)
    // Save `sepc`
    csrr t6, sepc
    sd t6, 0(sp)
    mv a0, sp
    csrr a1, scause
    csrr a2, stval
    // Check if floating point registers are dirty
    csrr s0, sstatus
    srli s0, s0, 13
    andi s0, s0, 3
    li s1, 3
    
    // Skip FP reg saving if they're clean
    bne s0, s1, 1f
    addi sp, sp, -264
    .attribute arch, "rv64imafdc"
    fsd f0, 0(sp)
    fsd f1, 8(sp)
    fsd f2, 16(sp)
    fsd f3, 24(sp)
    fsd f4, 32(sp)
    fsd f5, 40(sp)
    fsd f6, 48(sp)
    fsd f7, 56(sp)
    fsd f8, 64(sp)
    fsd f9, 72(sp)
    fsd f10, 80(sp)
    fsd f11, 88(sp)
    fsd f12, 96(sp)
    fsd f13, 104(sp)
    fsd f14, 112(sp)
    fsd f15, 120(sp)
    fsd f16, 128(sp)
    fsd f17, 136(sp)
    fsd f18, 144(sp)
    fsd f19, 152(sp)
    fsd f20, 160(sp)
    fsd f21, 168(sp)
    fsd f22, 176(sp)
    fsd f23, 184(sp)
    fsd f24, 192(sp)
    fsd f25, 200(sp)
    fsd f26, 208(sp)
    fsd f27, 216(sp)
    fsd f28, 224(sp)
    fsd f29, 232(sp)
    fsd f30, 240(sp)
    fsd f31, 248(sp)
    frcsr t1
    sd t1, 256(sp)
    .attribute arch, "rv64imac"
    li t1, (0b01 << 13)
    csrc sstatus, t1
    // FP registers clean
    1:
    call trap_handler
    // Check FP register status again
    bne s0, s1, 2f
    // Restore if they were dirty
    .attribute arch, "rv64imafdc"
    fld f0, 0(sp)
    fld f1, 8(sp)
    fld f2, 16(sp)
    fld f3, 24(sp)
    fld f4, 32(sp)
    fld f5, 40(sp)
    fld f6, 48(sp)
    fld f7, 56(sp)
    fld f8, 64(sp)
    fld f9, 72(sp)
    fld f10, 80(sp)
    fld f11, 88(sp)
    fld f12, 96(sp)
    fld f13, 104(sp)
    fld f14, 112(sp)
    fld f15, 120(sp)
    fld f16, 128(sp)
    fld f17, 136(sp)
    fld f18, 144(sp)
    fld f19, 152(sp)
    fld f20, 160(sp)
    fld f21, 168(sp)
    fld f22, 176(sp)
    fld f23, 184(sp)
    fld f24, 192(sp)
    fld f25, 200(sp)
    fld f26, 208(sp)
    fld f27, 216(sp)
    fld f28, 224(sp)
    fld f29, 232(sp)
    fld f30, 240(sp)
    fld f31, 248(sp)
    ld t1, 256(sp)
    fscsr t1
    .attribute arch, "rv64imac"
    addi sp, sp, 264
    // FP registers clean
    2:
    // Restore `sepc`
    ld t6, 0(sp)
    csrw sepc, t6
    // Reenable interrupts after sret (set SPIE)
    li t6, 1 << 5
    csrs sstatus, t6
    ld ra, 8(sp)
    // Skip sp for... obvious reasons
    ld gp, 24(sp)
    ld tp, 32(sp)
    ld t0, 40(sp)
    ld t1, 48(sp)
    ld t2, 56(sp)
    ld s0, 64(sp)
    ld s1, 72(sp)
    ld a0, 80(sp)
    ld a1, 88(sp)
    ld a2, 96(sp)
    ld a3, 104(sp)
    ld a4, 112(sp)
    ld a5, 120(sp)
    ld a6, 128(sp)
    ld a7, 136(sp)
    ld s2, 144(sp)
    ld s3, 152(sp)
    ld s4, 160(sp)
    ld s5, 168(sp)
    ld s6, 176(sp)
    ld s7, 184(sp)
    ld s8, 192(sp)
    ld s9, 200(sp)
    ld s10, 208(sp)
    ld s11, 216(sp)
    ld t3, 224(sp)
    ld t4, 232(sp)
    ld t5, 240(sp)
    ld t6, 248(sp)
    // Clear any outstanding atomic reservations
    sc.d zero, zero, 0(sp)
    // Restore `sp`
    ld sp, 16(sp)
    // gtfo
    sret
```