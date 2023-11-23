use crate::println;

#[repr(C)]
pub enum TrapCause {
    External(TrapExternal),
    Internal(TrapInternal),
}

#[repr(C)]
#[derive(Debug)]
pub enum TrapInternal {
    UnalignedAccess(AccessFault),
    InvalidAccess(AccessFault),
    PageFault(AccessFault),
    UnknownInstruction,
    Breakpoint,
    SystemCall
}

#[repr(C)]
pub enum TrapExternal {
    InterProcInt,
    Timer,
    ExternalDevice
}

#[derive(Debug)]
pub enum AccessFault {
    Load,
    Store,
    Exec,
}

pub fn trap_main(trapcause: TrapCause, regframe: &mut crate::arch::trap::TrapFrame) {
    match trapcause {
        TrapCause::External(cause) => {
            match cause {
                TrapExternal::Timer => {
                    crate::arch::timer::set_timer(u128::MAX);
                    crate::scheduler::next(regframe)
                }
                _ => todo!("Handle external interrupts"),
            }
        },
        TrapCause::Internal(cause) => {
            match cause {
                TrapInternal::PageFault(fault) => {
                    println!("Register frame dump {:#x?}", regframe);
                    page_fault(fault, regframe.pagefault_addr())
                },
                _ => panic!("Hit unhandled cause {:?} frame {:#x?} address 0x{:x}", cause, regframe, regframe.invalid_addr().addr())
            }
        }
    }
}

pub trait Frame {
    fn pagefault_addr(&self) -> crate::mem::VirtualAddress;
    fn invalid_addr(&self) -> crate::mem::VirtualAddress;
    fn unaligned_addr(&self) -> crate::mem::VirtualAddress;
}

pub fn page_fault(reason: AccessFault, vaddr: crate::mem::VirtualAddress) {
    let mut root = crate::arch::paging::get_root_table();

    let entry = root.get_entry(vaddr);
    match reason {
        AccessFault::Exec => assert!(entry.is_exec(), "Entry faulted but was not executable 0x{:x}", vaddr.addr()),
        AccessFault::Store => assert!(entry.is_write(), "Entry faulted but was not writeable 0x{:x}", vaddr.addr()),
        AccessFault::Load => assert!(entry.is_read(), "Entry faulted but was not readable 0x{:x}", vaddr.addr()),
    }

    if entry.swapped() {
        root.swap(vaddr, crate::cpu::THREAD_CTRL_BLOCK.lock().proc_id()).unwrap();
    } else if false {
        
    } else {
        panic!("Entry was invalid despite fault 0x{:x}", vaddr.addr())
    }
}