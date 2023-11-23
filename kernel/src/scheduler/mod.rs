use alloc::{collections::BTreeMap, sync::Arc};
use crossbeam_queue::SegQueue;
use spin::Mutex;

use crate::println;

pub fn exit_kthread() {
    use crate::arch::timer;
    timer::set_timer(timer::get_timer());
}

pub fn init_scheduler() {
    if PROC_LIST.lock().get(&0).is_none() {
        let thread_ids = vmem::Vmem::new(
            alloc::borrow::Cow::Borrowed("thread_ids"),
            1, 
            None
        );
        thread_ids.add(1, usize::MAX).unwrap();

        let kern_addr = vmem::Vmem::new(
            alloc::borrow::Cow::Borrowed("thread_ids"),
            4096, 
            Some(&crate::mem::VIRT)
        );

        let kernel_proc = Proc {
            thread_ids: Arc::new(thread_ids),
            address_space: Arc::new(kern_addr),
            proc_id: 0,
            page_table_addr: crate::arch::paging::get_root_table().addr()
        };
        PROC_LIST.lock().insert(0, Arc::new(kernel_proc));
    }
}

pub fn spawn_kernel_thread(f: fn() -> !, priority: i8) {
    let process = PROC_LIST.lock().get(&0).unwrap().clone();

    let addr_space = process.address_space.clone();
    let stack_addr = addr_space.alloc(0x10_0000, vmem::AllocStrategy::NextFit).unwrap();

    let mut root_table = unsafe {crate::arch::paging::RootTable::from_ptr(process.page_table_addr)};

    for i in (0..0x10_0000).step_by(0x1000) {
        let phys_addr = crate::mem::PHYS.alloc(0x1000, vmem::AllocStrategy::NextFit).unwrap();
        let vaddr = crate::mem::VirtualAddress::new(stack_addr + i);
        let paddr = crate::mem::PhysicalAddress::new(phys_addr);

        unsafe {
            root_table.map(
                vaddr, 
                paddr, 
                crate::arch::paging::PagePermissions::K_WRITE, 
                crate::arch::paging::PageSize::Kilopage
            ).unwrap();
        }
    }

    let mut trapframe = crate::arch::trap::TrapFrame::with_pc(f as usize);
    trapframe.set_stack(stack_addr);
    //trapframe.regs.gp = unsafe {crate::mem::linker::__global_pointer.as_usize()};
    //trapframe.regs.tp = crate::mem::tls::TLS.load(core::sync::atomic::Ordering::Relaxed);

    QUEUE.push(Thread {
        process: process.clone(),
        thread_id: process.thread_ids.alloc(1, vmem::AllocStrategy::NextFit).unwrap(),
        mode: crate::arch::Mode::Supervisor,
        trapframe,
        priority,
        priority_mod: 0
    });
    println!("Added kernel thread to queue");
}

pub fn next(frame: &mut crate::arch::trap::TrapFrame) {
    let mut cur_task = CUR_TASK.lock();

    let mut next_thread = QUEUE.pop().unwrap_or_else(|| {
        Thread {
            process: PROC_LIST.lock().get(&0).unwrap().clone(),
            thread_id: 0,
            mode: crate::arch::Mode::Supervisor,
            trapframe: crate::arch::trap::TrapFrame::with_pc(crate::arch::idle_thread as usize),
            priority: 1,
            priority_mod: 0,
        }
    });
    
    let timeshare = next_thread.time_share();

    if let Some(cur_thread) = cur_task.as_mut() {
        // Store the current frame
        cur_thread.trapframe = *frame;

        // Load new frame, mode, and page table
        next_thread.load_thread(frame);

        // Swap thread positions
        core::mem::swap(&mut next_thread, cur_thread);

        // rename old thread for clarity
        let old_thread = next_thread;

        // If old thread wasnt an idle thread, put it into the scheduler
        if old_thread.process.proc_id != 0 || old_thread.thread_id != 0 {
            QUEUE.push(old_thread);
        }
    } else {
        // Load new frame, mode, and page table
        next_thread.load_thread(frame);

        // Assign new thread
        *cur_task = Some(next_thread);
    }

    let timer = crate::arch::timer::get_timer();
    crate::arch::timer::set_timer(timer + timeshare);
}

#[thread_local]
static CUR_TASK: Mutex<Option<Thread>> = Mutex::new(None);

static QUEUE: SegQueue<Thread> = SegQueue::new();
static PROC_LIST: Mutex<BTreeMap<usize, Arc<Proc>>> = Mutex::new(BTreeMap::new());

struct Proc {
    thread_ids: Arc<vmem::Vmem<'static, 'static>>,
    address_space: Arc<vmem::Vmem<'static, 'static>>,
    proc_id: usize,
    page_table_addr: *mut crate::arch::paging::PageTable,
}

impl Proc {
    pub fn spawn_thread(self: Arc<Self>, mode: crate::arch::Mode, priority: i8, pc: usize) {
        let thread_id = self.thread_ids.alloc(1, vmem::AllocStrategy::NextFit).unwrap();
        let mut thread = Thread {
            process: self.clone(),
            thread_id,
            mode,
            trapframe: crate::arch::trap::TrapFrame::with_pc(pc),
            priority,
            priority_mod: 0,
        };

        let mut root_table = unsafe {crate::arch::paging::RootTable::from_ptr(thread.process.page_table_addr)};

        let addr_space = thread.process.address_space.clone();
        let stack_addr = addr_space.alloc(0x10_0000, vmem::AllocStrategy::NextFit).unwrap();

        for i in (0..0x10_0000).step_by(0x1000) {
            let phys_addr = crate::mem::PHYS.alloc(0x1000, vmem::AllocStrategy::NextFit).unwrap();
            let vaddr = crate::mem::VirtualAddress::new(stack_addr + i);
            let paddr = crate::mem::PhysicalAddress::new(phys_addr);

            let perms = match mode {
                crate::arch::Mode::Supervisor => crate::arch::paging::PagePermissions::K_WRITE,
                crate::arch::Mode::User => crate::arch::paging::PagePermissions::U_WRITE,
            };

            unsafe {
                root_table.map(
                    vaddr, 
                    paddr, 
                    perms, 
                    crate::arch::paging::PageSize::Kilopage
                ).unwrap();
            }
        }

        thread.trapframe.set_stack(stack_addr);

        QUEUE.push(thread);
    }
}

struct Thread {
    process: Arc<Proc>,
    thread_id: usize,
    mode: crate::arch::Mode,
    trapframe: crate::arch::trap::TrapFrame,
    priority: i8,
    priority_mod: i8,
}

impl Thread {
    fn load_thread(&self, trapframe: &mut crate::arch::trap::TrapFrame) {
        *trapframe = self.trapframe;
        let proc_lock = PROC_LIST.lock();

        let addr = proc_lock.get(&self.process.proc_id).as_ref().unwrap().page_table_addr;
        unsafe {
            crate::arch::paging::load_pagetable(addr);
            crate::arch::set_mode(self.mode);
        }
    }

    fn time_share(&self) -> u128 {
        let prior = self.priority + self.priority_mod;
        
        crate::arch::timer::ticks_from_ms(
            if prior <= 0 {
                0
            } else {
                (
                    prior as usize * prior as usize
                ) / 12
            } + 8
        )
    }
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

unsafe impl Send for Proc {}
unsafe impl Sync for Proc {}
