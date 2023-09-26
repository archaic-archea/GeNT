pub struct Allocator;

static HEAP: vmem::Vmem = vmem::Vmem::new(
    alloc::borrow::Cow::Borrowed("HEAP"), 
    1, 
    None
);

impl Allocator {
    pub unsafe fn init(&self, addr: *mut u8, size: usize) {
        HEAP.add(addr as usize, size).unwrap();
    }
}

unsafe impl core::alloc::GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let vmem_layout = vmem::Layout::new(layout.size());
        let vmem_layout = vmem_layout.align(layout.align());

        HEAP.alloc_constrained(vmem_layout, vmem::AllocStrategy::NextFit).unwrap() as *mut u8
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        let vmem_layout = vmem::Layout::new(layout.size());
        let vmem_layout = vmem_layout.align(layout.align());

        let alloc = HEAP.alloc_constrained(vmem_layout, vmem::AllocStrategy::NextFit).unwrap() as *mut u8;

        for i in 0..vmem_layout.size() {
            *alloc.add(i) = 0;
        }

        alloc
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        HEAP.free_constrained(ptr as usize, layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: core::alloc::Layout, new_size: usize) -> *mut u8 {
        let vmem_layout = vmem::Layout::new(new_size);
        let vmem_layout = vmem_layout.align(layout.align());

        let alloc = HEAP.alloc_constrained(vmem_layout, vmem::AllocStrategy::NextFit).unwrap() as *mut u8;

        for i in 0..layout.size() {
            *alloc.add(i) = *ptr.add(i);
        }

        HEAP.free_constrained(ptr as usize, layout.size());

        alloc
    }
}