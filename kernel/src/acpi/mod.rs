use alloc::alloc::{alloc, realloc, dealloc, Layout};

#[allow(warnings)]
pub mod raw_lai {
    include!(concat!(env!("OUT_DIR"), "/lai.rs"));
}

pub mod tables;

pub fn init_acpi() {
    let rsdp = crate::RSDP.response().unwrap().rsdp_addr as *mut tables::Rsdp;
    
    unsafe {
        let xsdt = (*rsdp).rsdt();
        xsdt.get_tables();
    }
}

pub struct Host;

fn layout(size: usize) -> Layout {
    unsafe {
        Layout::from_size_align_unchecked(size, core::mem::align_of::<usize>())
    }
}


impl lai::Host for Host {
    unsafe fn alloc(&self, size: usize) -> *mut u8 {
        alloc(
            layout(size)
        )
    }

    unsafe fn dealloc(&self, ptr: *mut u8, size: usize) {
        if ptr.is_null() {
            return;
        }

        dealloc(
            ptr, 
            layout(size)
        );
    }

    unsafe fn realloc(&self, ptr: *mut u8, new_size: usize, old_size: usize) -> *mut u8 {
        if ptr.is_null() {
            return alloc(
                layout(new_size)
            );
        }
        
        realloc(
            ptr, 
            layout(old_size), 
            new_size
        )
    }

    fn inb(&self, port: u16) -> u8 {
        let transit = unsafe {
            crate::arch::global::IOTransit::new(
                crate::arch::global::IOType::Port(port as usize)
            )
        };

        transit.read(0)
    }

    fn inw(&self, port: u16) -> u16 {
        let transit = unsafe {
            crate::arch::global::IOTransit::new(
                crate::arch::global::IOType::Port(port as usize)
            )
        };

        transit.read(0)
    }

    fn ind(&self, port: u16) -> u32 {
        let transit = unsafe {
            crate::arch::global::IOTransit::new(
                crate::arch::global::IOType::Port(port as usize)
            )
        };

        transit.read(0)
    }

    fn outb(&self, port: u16, value: u8) {
        let transit = unsafe {
            crate::arch::global::IOTransit::new(
                crate::arch::global::IOType::Port(port as usize)
            )
        };

        transit.write(0, value)
    }

    fn outw(&self, port: u16, value: u16) {
        let transit = unsafe {
            crate::arch::global::IOTransit::new(
                crate::arch::global::IOType::Port(port as usize)
            )
        };

        transit.write(0, value)
    }

    fn outd(&self, port: u16, value: u32) {
        let transit = unsafe {
            crate::arch::global::IOTransit::new(
                crate::arch::global::IOType::Port(port as usize)
            )
        };

        transit.write(0, value)
    }

    fn map(&self, paddr: usize, size: usize) ->  *mut u8 {
        let mut root_table = crate::arch::paging::get_root_table();
        let vaddr = crate::mem::VIRT.alloc(size, vmem::AllocStrategy::NextFit).unwrap();

        unsafe {
            for i in (0..size).step_by(4096) {
                let paddr = crate::mem::PhysicalAddress::new(paddr + i);
                let vaddr = crate::mem::VirtualAddress::new(vaddr + i);

                root_table.map(
                    vaddr, 
                    paddr, 
                    crate::arch::paging::PagePermissions {
                        read: true,
                        write: true,
                        execute: false,
                        user: false,
                        global: false,
                        dealloc: true
                    }, 
                    crate::arch::paging::PageSize::Kilopage
                ).unwrap();
            }
        }

        vaddr as *mut u8
    }

    fn unmap(&self, vaddr: usize, size: usize) {
        let mut root_table = crate::arch::paging::get_root_table();
        
        unsafe {
            for i in (0..size).step_by(4096) {
                let vaddr = crate::mem::VirtualAddress::new(vaddr + i);

                root_table.unmap(
                    vaddr, 
                    crate::arch::paging::PageSize::Kilopage
                ).unwrap();
            }
        }
    }

    fn pci_readb(&self,_seg:u16,_bus:u8,_slot:u8,_fun:u8,_offset:u16) -> u8 {
        todo!()
    }
    
    fn pci_readw(&self,_seg:u16,_bus:u8,_slot:u8,_fun:u8,_offset:u16) -> u16 {
        todo!()
    }

    fn pci_readd(&self,_seg:u16,_bus:u8,_slot:u8,_fun:u8,_offset:u16) -> u32 {
        todo!()
    }

    fn pci_writeb(&self, _seg: u16, _bus: u8, _slot: u8, _fun: u8, _offset: u16, _value: u8) {
        todo!()
    }

    fn pci_writew(&self, _seg: u16, _bus: u8, _slot: u8, _fun: u8, _offset: u16, _value: u16) {
        todo!()
    }

    fn pci_writed(&self, _seg: u16, _bus: u8, _slot: u8, _fun: u8, _offset: u16, _value: u32) {
        todo!()
    }

    fn timer(&self) -> u64 {
        todo!()
    }

    fn scan(&self, signature: &str, _index: usize) ->  *mut u8 {
        /*let rsdp = crate::RSDP.response().unwrap().rsdp_addr as *mut tables::Rsdp;
        println!("RSDP {:?}", rsdp);
        
        unsafe {
            let xsdt = (*rsdp).rsdt();
            
            xsdt.get_table(signature) as *mut u8
        }*/

        let sig = [signature.as_bytes()[0], signature.as_bytes()[1], signature.as_bytes()[2], signature.as_bytes()[3]];

        let lock = tables::LOOKUP_TABLE.lock();
        let lookup = lock.get(&sig);

        if lookup.is_none() {
            return core::ptr::null_mut();
        }

        (*lookup.unwrap() as *const tables::SdtHeader).cast_mut() as *mut u8
    }

    fn sleep(&self,_ms:u64) {
        todo!()
    }
}