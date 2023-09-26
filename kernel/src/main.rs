#![no_main]
#![no_std]
#![feature(
    naked_functions,
    int_roundings,
    strict_provenance,
    pointer_byte_offsets,
    fn_align
)]

extern crate alloc;

use alloc::format;
use gent_kern::{println, print};
use gent_kern::acpi;

static MMAP: limine::MemoryMapRequest = limine::MemoryMapRequest::new();

#[no_mangle]
extern "C" fn kinit() -> ! {
    log::set_logger(&gent_kern::uart::Logger).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    println!("Starting kernel");
    println!("Mode: {:?}", gent_kern::MODE.response().unwrap().mode());
    unsafe {
        vmem::bootstrap()
    }

    println!("vmem bootstrapped");

    let memory_map = MMAP.response().expect("Failed to get map");
    for entry in memory_map.usable_entries() {
        gent_kern::mem::PHYS.add(entry.base, entry.size).expect("Failed to add entry");
    }

    println!("Memory map fetched");

    gent_kern::find_upperhalf_mem();
    println!("Upperhalf found");

    gent_kern::mem::tls::init_tls();
    gent_kern::arch::trap::init_traps();
    println!("Traps initialized");

    gent_kern::allocator::init();
    println!("Memory initialized");

    gent_kern::dev::blockdev::init();

    let host = alloc::sync::Arc::new(gent_kern::acpi::Host);

    lai::init(host);
    lai::create_namespace();

    println!("Made LAI namespace");

    print_nodes(0, lai::get_root());

    let rsdp = unsafe {&*{gent_kern::RSDP.response().unwrap().rsdp_addr as *mut acpi::tables::Rsdp}};
    let rsdt = rsdp.rsdt();

    for node in rsdt.into_iter() {
        println!("Found node {:#?}", node.name());
    }
    
    let madt = unsafe {&*(rsdt.get_table("APIC") as *const acpi::tables::madt::Madt)};

    let intcontroller = acpi::tables::madt::RiscvIntController::from_entry(
        madt.entry(0).unwrap()
    ).unwrap();

    //let mut fw_cfg = lai::resolve_path(None, "\\_SB_.FWCF._CRS").unwrap();
    //let fw_cfg = fw_cfg.eval().unwrap();
    //println!("Evaluated FW_CFG node");
    //let buf = fw_cfg.get_buffer().unwrap();
    //println!("FW_CFG addr: {:?} len {}", buf.as_ptr(), buf.len());

    /*for i in 0..8 {
        let mut virtio = lai::resolve_path(None, &format!("\\_SB_.VR0{}._CRS", i)).unwrap();
        let mmio = virtio.eval().unwrap();
        let mmio = mmio.get_buffer().unwrap();
    
        assert!(mmio[0] == 0b10000110);
        assert!(mmio[1] == 0x09);
        assert!(mmio[2] == 0x00);
    
        let base: *mut gent_kern::dev::virtio::VirtIoHeader = gent_kern::mem::PhysicalAddress::new(
            u32::from_le_bytes([mmio[4], mmio[5], mmio[6], mmio[7]]) as usize
        ).to_virt().to_mut_ptr();
    
        let virtio = unsafe {gent_kern::dev::virtio::VirtIoHeader::from_mut_ptr(base).unwrap()};
        println!("Found device {:?}", virtio.dev_id());
    }*/

    /*let aml_handler: Box<dyn aml::Handler> = Box::new(gent_kern::arch::transit::Transit);

    let mut context = aml::AmlContext::new(
        aml_handler, 
        aml::DebugVerbosity::All
    );
    println!("Made AML context");

    unsafe {
        let dsdt = xsdt.dsdt().unwrap();
        context.parse_table(
            &*core::ptr::slice_from_raw_parts(
                dsdt.address as *const u8, 
                dsdt.length as usize
            )
        ).unwrap();

        for ssdt in xsdt.ssdts() {
            context.parse_table(
                &*core::ptr::slice_from_raw_parts(
                    ssdt.address as *const u8, 
                    ssdt.length as usize
                )
            ).unwrap();
        }

        context.initialize_objects().unwrap();
    }

    println!("{:#?}", context.namespace);

    let resources = aml::resource::resource_descriptor_list(
        context.namespace.get_by_path(
            &aml::AmlName::from_str("\\_SB_.FWCF._CRS").unwrap()
        ).unwrap()
    ).unwrap();

    let fw_cfg = &resources[0];

    match fw_cfg {
        aml::resource::Resource::MemoryRange(range) => {
            match range {
                aml::resource::MemoryRangeDescriptor::FixedLocation { 
                    is_writable: _, 
                    base_address, 
                    range_length : _
                } => {
                    let base_address = (*base_address as usize) + gent_kern::mem::HHDM_OFFSET.load(core::sync::atomic::Ordering::Relaxed);

                    // TODO: Figure out how to see if something is Port IO, or Memory IO
                    let fw_cfg = unsafe {gent_kern::dev::fw_cfg::FwCfg::new(gent_kern::arch::global::IOType::Mem(base_address))};

                    let files = fw_cfg.files();

                    for file in files.iter() {
                        println!("File {:#?}", file.name().unwrap());
                    }

                    let ramfb = fw_cfg.lookup("etc/ramfb").unwrap();

                    println!("RAMFB sel: 0x{:x}", ramfb.sel().get());

                    let ramfbcfg = gent_kern::dev::ramfb::RAMFBConfig::new(640, 480);
                    let cfg_arr: [u8; 28] = unsafe {core::mem::transmute(ramfbcfg)};

                    fw_cfg.write_file(ramfb, &cfg_arr).unwrap();

                    let ramfbcfg: gent_kern::dev::ramfb::RAMFBConfig = unsafe {core::mem::transmute(cfg_arr)};

                    for i in 0..ramfbcfg.byte_size() {
                        unsafe {
                            let ptr = ramfbcfg.addr() as *mut u8;

                            ptr.add(i).write_volatile(0xff);
                        }
                    }
                }
            }
        },
        _ => unreachable!()
    }*/

    /*let phys = gent_kern::mem::PHYS.alloc(4096, vmem::AllocStrategy::NextFit).unwrap();
    let virt = gent_kern::mem::VIRT.alloc(4096, vmem::AllocStrategy::NextFit).unwrap();

    println!("Allocated swappable RAM at virt 0x{:x} phys 0x{:x}", virt, phys);

    let mut root = gent_kern::arch::paging::get_root_table();
    unsafe {
        root.map(
            gent_kern::mem::VirtualAddress::new(virt), 
            gent_kern::mem::PhysicalAddress::new(phys), 
            gent_kern::arch::paging::PagePermissions {
                read: true,
                write: true,
                execute: false,
                user: false,
                global: false,
                dealloc: true,
            }, 
            gent_kern::arch::paging::PageSize::Kilopage
        ).unwrap();

        *(virt as *mut u8) = 243;

        root.get_entry(
            gent_kern::mem::VirtualAddress::new(virt)
        ).set_perms(gent_kern::arch::paging::PagePermissions {
            read: true,
            write: false,
            execute: false,
            user: false,
            global: false,
            dealloc: true,
        });
    }

    println!("Swapping out 0x{:x}", virt);

    root.swap(
        gent_kern::mem::VirtualAddress::new(virt), 
        0
    ).unwrap();

    unsafe {
       println!("{}", *(virt as *mut u8));
       println!("Writing (should generate a `Store Page Fault`)");
       (virt as *mut u8).write_volatile(0);
       println!("{}", *(virt as *mut u8));
    }*/

    panic!("Kernel end");
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {:#?}", info);
    loop {
        gent_kern::arch::utils::slow();
    }
}

fn print_nodes(tabs: usize, node: lai::Node) {
    print!("{}-└{} {:?}", "  ".repeat(tabs), node.name(), node.object().typ());

    if node.object().typ() == lai::ObjectType::String {
        println!(" {:#?}", node.object().get_str().unwrap());
    } else {
        println!()
    }

    for node in node.into_iter() {
        print_nodes(tabs + 1, node);
    }
}