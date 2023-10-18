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
    pointer_byte_offsets,
    repr128
)]

extern crate alloc;

pub mod allocator;
pub mod arch;
pub mod mem;
pub mod uart;
pub mod dma; 
pub mod dev;
mod scheduler;
pub mod acpi;
mod utils;
mod cpu;

static KERN_FILE: limine::KernelFileRequest = limine::KernelFileRequest::new();
pub static RSDP: limine::RsdpRequest = limine::RsdpRequest::new();
static HHDM: limine::HhdmRequest = limine::HhdmRequest::new();
pub static MODE: limine::PagingModeRequest = limine::PagingModeRequest::new(
    limine::PagingMode::Sv57, 
    limine::PagingModeRequestFlags::empty()
);

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