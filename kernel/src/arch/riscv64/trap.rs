use crate::println;

#[repr(C)]
struct Sscratch {
    ksp: usize,
    ktp: usize,
    kgp: usize,
    stack_scratch: usize,
}

#[thread_local]
static mut SSCRATCH: Sscratch = Sscratch {
    kgp: 0,
    ksp: 0,
    ktp: 0,
    stack_scratch: 0,
};

pub fn init_traps() {
    unsafe {
        const STACK_SIZE: usize = 0x1000;

        println!("Initializing traps");
        SSCRATCH.kgp = crate::mem::linker::__global_pointer.as_usize();
        println!("Loaded GP");
        
        let virt = crate::mem::VIRT.alloc(STACK_SIZE, vmem::AllocStrategy::NextFit).unwrap();
        let phys = crate::mem::PHYS.alloc(STACK_SIZE, vmem::AllocStrategy::NextFit).unwrap();

        let mut table = super::paging::get_root_table();

        for i in (0..STACK_SIZE).step_by(4096) {
            let vaddr = crate::mem::VirtualAddress::new(virt + i);
            let paddr = crate::mem::PhysicalAddress::new(phys + i);

            table.map(
                vaddr, 
                paddr, 
                super::paging::PagePermissions {
                    read: true,
                    write: true,
                    execute: false,
                    user: false,
                    global: true,
                    dealloc: false,
                }, 
                super::paging::PageSize::Kilopage
            ).unwrap();
        }

        SSCRATCH.ksp = virt;
        println!("Loaded SP");

        SSCRATCH.ktp = crate::mem::tls::TLS.load(core::sync::atomic::Ordering::Relaxed);
        println!("Loaded TP");
        
        let ptr = core::ptr::addr_of!(SSCRATCH) as usize;
        println!("Made pointer");

        core::arch::asm!(
            "csrw sscratch, {sscratch}",
            sscratch = in(reg) ptr
        );
    }
}

extern "C" fn trap_riscv_main(trapframe: TrapFrame, scause: Cause) {
    use crate::arch::global::trap::{self, TrapCause, TrapInternal, TrapExternal, AccessFault};

    let trapcause = match scause {
        Cause::Breakpoint => TrapCause::Internal(TrapInternal::Breakpoint),
        Cause::IllegalInstr => TrapCause::Internal(TrapInternal::UnknownInstruction),
        Cause::LoadPageFault => TrapCause::Internal(TrapInternal::PageFault(AccessFault::Load)),
        Cause::StorePageFault => TrapCause::Internal(TrapInternal::PageFault(AccessFault::Store)),
        Cause::InstrPageFault => TrapCause::Internal(TrapInternal::PageFault(AccessFault::Exec)),
        Cause::UnalignedLoad => TrapCause::Internal(TrapInternal::UnalignedAccess(AccessFault::Load)),
        Cause::UnalignedStore => TrapCause::Internal(TrapInternal::UnalignedAccess(AccessFault::Store)),
        Cause::UnalignedInst => TrapCause::Internal(TrapInternal::UnalignedAccess(AccessFault::Exec)),
        Cause::InvalidLoad => TrapCause::Internal(TrapInternal::InvalidAccess(AccessFault::Load)),
        Cause::InvalidStore => TrapCause::Internal(TrapInternal::InvalidAccess(AccessFault::Store)),
        Cause::InvalidInstrAddr => TrapCause::Internal(TrapInternal::InvalidAccess(AccessFault::Exec)),
        Cause::Ecall => TrapCause::Internal(TrapInternal::SystemCall),

        Cause::ExtInt => TrapCause::External(TrapExternal::ExternalDevice),
        Cause::InterProcInt => TrapCause::External(TrapExternal::InterProcInt),
        Cause::TimerInt => TrapCause::External(TrapExternal::Timer),

        Cause::PlatformInt => panic!("Platform specific interrupts not supported"),
    };

    trap::trap_main(
        trapcause, 
        trapframe
    )
}

#[repr(u64)]
#[derive(Debug)]
pub enum Cause {
    UnalignedInst = 0x0,
    InvalidInstrAddr = 0x1,
    IllegalInstr = 0x2,
    Breakpoint = 0x3,
    UnalignedLoad = 0x4,
    InvalidLoad = 0x5,
    UnalignedStore = 0x6,
    InvalidStore = 0x7,
    Ecall = 0x8,
    InstrPageFault = 0xC,
    LoadPageFault = 0xD,
    StorePageFault = 0xF,
    InterProcInt = 0x8000000000000001,
    TimerInt = 0x8000000000000005,
    ExtInt = 0x8000000000000009,
    PlatformInt,
}

#[naked]
#[no_mangle]
#[repr(align(4))]
pub(crate) unsafe extern "C" fn stvec_trap_shim() -> ! {
    core::arch::asm!(r#"
        // Swap sscratch and t0
        csrrw t0, sscratch, t0

        // Swap stack pointers
        sd sp, 24(t0)
        ld sp, 8(t0)

        // Increase stack pointer for the registers
        addi sp, sp, {TRAP_FRAME_SIZE}

        // Save GP, and TP first
        sd gp, 16(sp)
        sd tp, 24(sp)

        // Store user stack onto kernel stack
        ld gp, 24(t0)
        sd gp, 8(sp)
        
        // Load kernel gp and tp
        ld gp, 0(t0)
        ld tp, 16(t0)

        // Sscratch isnt needed anymore, we can swap it back
        csrrw t0, sscratch, t0

        // Save the rest of the registers
        sd ra, 0(sp)
        // sp was saved at 8(sp)
        // gp was saved at 16(sp)
        // tp was saved at 24(sp)
        sd t0, 32(sp)
        sd t1, 40(sp)
        sd t2, 48(sp)
        sd s0, 56(sp)
        sd s1, 64(sp)
        sd a0, 72(sp)
        sd a1, 80(sp)
        sd a2, 88(sp)
        sd a3, 96(sp)
        sd a4, 104(sp)
        sd a5, 112(sp)
        sd a6, 120(sp)
        sd a7, 128(sp)
        sd s2, 136(sp)
        sd s3, 144(sp)
        sd s4, 152(sp)
        sd s5, 160(sp)
        sd s6, 168(sp)
        sd s7, 176(sp)
        sd s8, 184(sp)
        sd s9, 192(sp)
        sd s10, 200(sp)
        sd s11, 208(sp)
        sd t3, 216(sp)
        sd t4, 224(sp)
        sd t5, 232(sp)
        sd t6, 240(sp)

        // Save sepc
        csrr t0, sepc
        sd t0, 248(sp)

        // Enter args for the main RISC-V trap function
        mv a0, sp
        csrr a1, scause

        // Check floating point registers
        csrr s0, sstatus
        srli s0, s0, 13
        andi s0, s0, 3
        li s1, 3

        // Skip FP reg saving if they're clean
        bne s0, s1, 1f

            // Save floating point registers
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

            // Save the floating point flags
            frcsr t1
            sd t1, 256(sp)

            // Clear dirty floating point bits
            .attribute arch, "rv64imac"
            li t1, (0b01 << 13)
            csrc sstatus, t1

            // Floating point registers are clean
            1:

        call {trap}

        // Load back into userspace

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

        // Load sepc
        ld t0, 248(sp)

        csrw sepc, t0

        // Load user registers
        ld ra, 0(sp)
        ld gp, 16(sp)
        ld tp, 24(sp)
        ld t0, 32(sp)
        ld t1, 40(sp)
        ld t2, 48(sp)
        ld s0, 56(sp)
        ld s1, 64(sp)
        ld a0, 72(sp)
        ld a1, 80(sp)
        ld a2, 88(sp)
        ld a3, 96(sp)
        ld a4, 104(sp)
        ld a5, 112(sp)
        ld a6, 120(sp)
        ld a7, 128(sp)
        ld s2, 136(sp)
        ld s3, 144(sp)
        ld s4, 152(sp)
        ld s5, 160(sp)
        ld s6, 168(sp)
        ld s7, 176(sp)
        ld s8, 184(sp)
        ld s9, 192(sp)
        ld s10, 200(sp)
        ld s11, 208(sp)
        ld t3, 216(sp)
        ld t4, 224(sp)
        ld t5, 232(sp)
        ld t6, 240(sp)

        // SP is loaded last to prevent being incapable of accessing data
        ld sp, 8(sp)

        sret
    "#, 
    trap = sym trap_riscv_main,
    TRAP_FRAME_SIZE = const { -(core::mem::size_of::<TrapFrame>() as isize) }, 
    options(noreturn));
}

#[repr(C)]
pub struct GeneralRegisters {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
}

#[repr(C)]
pub struct TrapFrame {
    regs: GeneralRegisters,
    sepc: usize
}

impl crate::arch::global::trap::Frame for TrapFrame {
    fn invalid_addr(&self) -> crate::mem::VirtualAddress {
        super::csr::Stval::new().addr()
    }

    fn pagefault_addr(&self) -> crate::mem::VirtualAddress {
        super::csr::Stval::new().addr()
    }

    fn unaligned_addr(&self) -> crate::mem::VirtualAddress {
        super::csr::Stval::new().addr()
    }
}