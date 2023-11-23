use crate::println;

pub mod transit;
pub mod paging;
pub mod trap;
pub mod mem;
mod csr;
mod boot;
pub mod timer;
pub mod utils;

pub fn init() {
    // Relocate the global section to 0 tbh
    let mut roottable = super::paging::get_root_table();
    
    unsafe {
        roottable.remove_entries(0..256);

        for (i, gp) in (crate::mem::linker::__data_start.as_usize()..crate::mem::linker::__data_end.as_usize()).step_by(0x1000).enumerate() {
            let i = i * 0x1000;
            let entry = roottable.get_entry(crate::mem::VirtualAddress::new(gp));
            roottable.map(
                crate::mem::VirtualAddress::new(i), 
                entry.phys(), 
                super::paging::PagePermissions {
                    read: true,
                    write: gp >= crate::mem::linker::__wdata_start.as_usize(),
                    execute: false,
                    user: false,
                    global: false,
                    dealloc: false,
                }, 
                super::paging::PageSize::Kilopage
            ).unwrap();
        }
    }

    let table = (*crate::acpi::tables::LOOKUP_TABLE.lock().get(b"RHCT").unwrap()) as *const _;
    let table = unsafe {&*(table as *const crate::acpi::tables::Rhct)};
    super::timer::FREQ.store(table.timer_freq as usize, core::sync::atomic::Ordering::Relaxed)
}

pub fn set_mode(mode: super::Mode) {
    let spp = match mode {
        super::Mode::User => false,
        super::Mode::Supervisor => true,
    };

    let mut sstatus = csr::Sstatus::default();
    sstatus.set_spp(spp);
    unsafe {sstatus.load()};
}

#[naked]
pub extern "C" fn idle_thread() -> ! {
    unsafe {core::arch::asm!(
        "
            1:
                wfi
                j 1b
        ",
        options(noreturn)
    )}
}

pub const _PRINT: fn(args: core::fmt::Arguments) = sbi_write;

fn sbi_write(args: core::fmt::Arguments) {
    use core::fmt::Write;

    let mut console = SbiConsole;
    console.write_fmt(args).unwrap();
}

struct SbiConsole;

impl core::fmt::Write for SbiConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let vaddr = mem::VirtualAddress::new(s.as_ptr() as usize);
        let paddr = paging::get_root_table().get_entry(vaddr).phys();
        let paddr = paddr.addr() + (vaddr.addr() & 0xfff);

        unsafe {
            sbi::ecall3(s.len(), paddr, 0, 0x4442434E, 0).unwrap_or_else(|err| { loop {} });
        }

        Ok(())
    }
}