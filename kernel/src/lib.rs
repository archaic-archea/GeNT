#![no_std]
#![feature(
    int_roundings,
    new_uninit,
    strict_provenance,
    alloc_error_handler,
    naked_functions,
    fn_align,
    thread_local,
    asm_const,
    stdsimd,
    riscv_ext_intrinsics,
    repr128,
    c_str_literals,
    const_mut_refs,
)]

use spin::Mutex;

extern crate alloc;

pub mod allocator;
pub mod arch;
pub mod mem;
pub mod dma; 
pub mod dev;
pub mod scheduler;
pub mod acpi;
pub mod object;
mod utils;
mod cpu;

pub static FBREQ: limine::FramebufferRequest = limine::FramebufferRequest::new();
static KERN_FILE: limine::KernelFileRequest = limine::KernelFileRequest::new();
pub static RSDP: limine::RsdpRequest = limine::RsdpRequest::new();
static HHDM: limine::HhdmRequest = limine::HhdmRequest::new();
pub static MODE: limine::PagingModeRequest = limine::PagingModeRequest::new(
    limine::PagingMode::Sv57, 
    limine::PagingModeRequestFlags::empty()
);

pub fn parse_kern_file() {
    let file = crate::KERN_FILE.response().unwrap();

    let elf: elf::ElfBytes<'_, elf::endian::NativeEndian> = elf::ElfBytes::minimal_parse(file.data()).unwrap();
    let table: elf::segment::SegmentTable<elf::endian::NativeEndian> = elf.segments().unwrap();
    
    for phdr in table.iter() {
        let phdr: elf::segment::ProgramHeader = phdr;
        if phdr.p_type == elf::abi::PT_TLS {
            mem::tls::init_tls(phdr);
        }
    }
}

pub fn find_upperhalf_mem() {
    let base = HHDM.response().unwrap().base;

    mem::HHDM_OFFSET.store(
        base,
        core::sync::atomic::Ordering::Relaxed
    );

    let root = arch::paging::get_root_table();


    let mut addr = base;

    while (addr < 0xffffffff80000000) && (addr >= base) {
        let vaddr = mem::VirtualAddress::new(addr);

        let (entry, level) = root.read(vaddr);

        let size = arch::paging::PageSize::from_level(level) as usize;

        match entry {
            arch::paging::Entry::Invalid => {
                mem::VIRT.add(addr, size).expect("Failed to add virtual entry");
            },
            arch::paging::Entry::Page(_) => {},
            arch::paging::Entry::Table(_) => unreachable!()
        }

        addr += size;
    }
}

static PRINTFN: Mutex<fn(args: core::fmt::Arguments)> = Mutex::new(arch::_PRINT);

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!{"{}\n", format_args!($($arg)*)});
}
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    crate::arch::trap::disable();
    PRINTFN.lock()(args);
    crate::arch::trap::enable();
}

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn flush(&self) {
    }

    fn log(&self, record: &log::Record) {
        println!("[{}]{}: {}", record.level(), record.target(), record.args());
    }
}

#[macro_export]
macro_rules! spin_loop {
    ($id:expr) => {
        while $id.compare_exchange_weak(false, true, core::sync::atomic::Ordering::AcqRel, core::sync::atomic::Ordering::Acquire).is_err() {
            core::hint::spin_loop();
        }
    };
}


#[macro_export]
macro_rules! drop_spin {
    ($id:expr) => {
        $id.store(false, core::sync::atomic::Ordering::Release);
    };
}