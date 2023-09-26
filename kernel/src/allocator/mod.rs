//use linked_list_allocator::LockedHeap;

mod vmem_alloc;

#[global_allocator]
//static ALLOCATOR: LockedHeap = LockedHeap::empty();
static ALLOCATOR: vmem_alloc::Allocator = vmem_alloc::Allocator;

pub fn init() {
    const HEAP_SIZE: usize = 0x100000;

    let physalloc = crate::mem::PHYS.alloc(HEAP_SIZE, vmem::AllocStrategy::NextFit).unwrap();
    let alloc = crate::mem::VIRT.alloc(HEAP_SIZE, vmem::AllocStrategy::NextFit).unwrap();

    let mut root_table = crate::arch::paging::get_root_table();
    let perms = crate::arch::paging::PagePermissions {
        read: true,
        write: true,
        execute: false,
        user: false,
        global: true,
        dealloc: false,
    };

    // Map the heap
    for i in (0..HEAP_SIZE).step_by(4096) {
        let vaddr = crate::mem::VirtualAddress::new(alloc + i);
        let paddr = crate::mem::PhysicalAddress::new(physalloc + i);

        unsafe {
            root_table.map(
                vaddr, 
                paddr, 
                perms, 
                crate::arch::paging::PageSize::Kilopage
            ).unwrap();
        }
    }

    unsafe {
        ALLOCATOR.init(alloc as *mut u8, HEAP_SIZE);
    }
}

#[alloc_error_handler]
fn mem_panic(info: core::alloc::Layout) -> ! {
    panic!("{:#?}", info);
}