use core::sync::atomic::AtomicUsize;

use elf::segment::ProgramHeader;

#[thread_local]
pub static TLS: AtomicUsize = AtomicUsize::new(0);

/// Call once per thread
pub fn init_tls(phdr: ProgramHeader) {
    let file = crate::KERN_FILE.response().unwrap();

    /*let elf: elf::ElfBytes<'_, elf::endian::NativeEndian> = elf::ElfBytes::minimal_parse(file.data()).unwrap();
    let table: elf::segment::SegmentTable<elf::endian::NativeEndian> = elf.segments().unwrap();

    let mut new_tls: usize = 0;
    
    for phdr in table.iter() {
        let phdr: elf::segment::ProgramHeader = phdr;
        if phdr.p_type == elf::abi::PT_TLS {*/
    let layout = vmem::Layout::new(phdr.p_memsz as usize).align(phdr.p_align as usize);
    let phys = super::PHYS.alloc_constrained(layout, vmem::AllocStrategy::NextFit).unwrap();
    let virt = super::VIRT.alloc_constrained(layout, vmem::AllocStrategy::NextFit).unwrap();
    let mut root_table = crate::arch::paging::get_root_table();

    let new_tls = virt;

    unsafe {
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

        let new_tls = core::slice::from_raw_parts_mut(virt as *mut u8, phdr.p_memsz as usize);
        let tls_file = &file.data()[file_base..file_base + file_size];

        for byte in &mut new_tls[..] {
            *byte = 0;
        }

        core::ptr::copy(tls_file.as_ptr(), new_tls.as_mut_ptr(), tls_file.len());
    }
        //}
    //}

    unsafe {core::arch::asm!("mv tp, {tp}", tp = in(reg) new_tls);}
    TLS.store(new_tls, core::sync::atomic::Ordering::Relaxed);
}