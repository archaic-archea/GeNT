use core::sync::atomic::AtomicUsize;

use crate::println;

#[thread_local]
pub static TLS: AtomicUsize = AtomicUsize::new(0);

/// Call once per thread
pub fn init_tls() {
    let file = crate::KERN_FILE.response().unwrap();

    let elf: elf::ElfBytes<'_, elf::endian::NativeEndian> = elf::ElfBytes::minimal_parse(file.data()).unwrap();
    let table: elf::segment::SegmentTable<elf::endian::NativeEndian> = elf.segments().unwrap();

    let mut new_tls: usize = 0;
    
    for phdr in table.iter() {
        let phdr: elf::segment::ProgramHeader = phdr;
        if phdr.p_type == elf::abi::PT_TLS {
            println!("Found TLS");

            let phys = super::PHYS.alloc(phdr.p_memsz as usize, vmem::AllocStrategy::NextFit).unwrap();
            let virt = super::VIRT.alloc(phdr.p_memsz as usize, vmem::AllocStrategy::NextFit).unwrap();
            let mut root_table = crate::arch::paging::get_root_table();

            new_tls = virt;

            unsafe {
                println!("Mapping TLS");
                for i in (0..phdr.p_memsz as usize).step_by(4096) {
                    let phys = super::PhysicalAddress::new(phys + i);
                    let virt = super::VirtualAddress::new(virt + i);
        
                    root_table.map(
                        virt, 
                        phys, 
                        crate::arch::paging::PagePermissions {
                            read: true,
                            write: true,
                            execute: false,
                            user: false,
                            global: false,
                            dealloc: false,
                        }, 
                        crate::arch::paging::PageSize::Kilopage
                    ).unwrap();
                }

                let file_base = phdr.p_offset as usize;
                let file_size = phdr.p_filesz as usize;

                println!("Getting TLS slices");
                let tls_slice = core::slice::from_raw_parts_mut(virt as *mut u8, phdr.p_memsz as usize);
                let tls_file_slice = &file.data()[file_base..file_base + file_size];


                println!("Writing TLS");
                for (index, byte) in tls_file_slice.iter().enumerate() {
                    tls_slice[index] = *byte;
                }

                println!("Clearing TBSS");
                for byte in &mut tls_slice[file_size..] {
                    *byte = 0;
                }
            }
        }
    }

    println!("Loading TLS from 0x{:x}", new_tls);
    unsafe {core::arch::asm!("mv tp, {tp}", tp = in(reg) new_tls);}
    println!("Storing TLS to {:?}", TLS.as_ptr());
    TLS.store(new_tls, core::sync::atomic::Ordering::Relaxed);
}