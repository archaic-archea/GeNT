#![no_main]
#![no_std]
#![feature(
    naked_functions,
    int_roundings,
    strict_provenance,
    fn_align
)]

extern crate alloc;

use gent_kern::{println, print};
use gent_kern::acpi;

static MMAP: limine::MemoryMapRequest = limine::MemoryMapRequest::new();

#[no_mangle]
extern "C" fn kinit() -> ! {
    log::set_logger(&gent_kern::Logger).unwrap();
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

    gent_kern::allocator::init();
    println!("Memory initialized");

    gent_kern::acpi::init_acpi();

    gent_kern::arch::init();

    gent_kern::parse_kern_file();
    gent_kern::arch::trap::init_traps();
    println!("Traps initialized");

    gent_kern::scheduler::init_scheduler();

    gent_kern::dev::window::init();

    for fb in gent_kern::FBREQ.response().unwrap().framebuffers() {
        println!("Adding framebuffer {:?}", fb);
        gent_kern::dev::window::DISPLAY_QUEUE.push(
            gent_kern::dev::window::Request::AddFrameBuffer(
                fb.addr as *mut u32, 
                fb.width as usize, 
                fb.height as usize, 
                fb.stride as usize
            )
        )
    }

    gent_kern::dev::blockdev::init();
    println!("Block device initialized");

    let host = alloc::sync::Arc::new(gent_kern::acpi::Host);

    lai::init(host);
    lai::create_namespace();

    println!("LAI initialized");

    let device_iter = lai::resolve_path(None, "\\_SB_").unwrap();

    // Store driver refs temporarily
    let mut btree = alloc::collections::BTreeMap::new();

    for driver in &gent_kern::dev::DRIVERS {
        println!("Found driver {}", driver.id);
        btree.insert(driver.id, driver.init);
    }

    for dev in device_iter.into_iter() {
        if let Some(hid) = dev.child("_HID") {
            let hid = hid.object().get_str().unwrap();
            println!("Found device with HID {:?}", hid);
            
            if let Some(driver) = btree.get(hid) {
                driver(dev);
            }
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

    print_nodes(0, lai::get_root());

    /*let rsdp = unsafe {&*{gent_kern::RSDP.response().unwrap().rsdp_addr as *mut acpi::tables::Rsdp}};
    let rsdt = rsdp.rsdt();

    for node in rsdt.into_iter() {
        println!("Found node {:#?}", node.name());
    }
    
    let madt_ptr = (*acpi::tables::LOOKUP_TABLE.lock().get(b"APIC").unwrap() as *const acpi::tables::SdtHeader).cast_mut() as *mut acpi::tables::madt::Madt;
    gent_kern::arch::trap::MADT.store(madt_ptr, core::sync::atomic::Ordering::Relaxed);
    let madt = unsafe {&*madt_ptr};

    let iter = madt.iter();

    for entry in iter {
        use acpi::tables::madt::{EntryType, self};

        match entry.etype() {
            EntryType::RiscvIntController => {
                let ctrl = madt::RiscvIntController::from_entry(entry).unwrap();

                if !ctrl.supported() {
                    println!("Unsupported RINTC");
                    break;
                }
                println!("Found supported {:#x?}", ctrl);
            },
            EntryType::Imsic => {
                let ctrl = madt::Imsic::from_entry(entry).unwrap();

                if !ctrl.supported() {
                    println!("Unsupported IMSIC");
                    break;
                }

                println!("Found supported {:#x?}", ctrl);
            },
            EntryType::Aplic => {
                let ctrl = madt::Aplic::from_entry(entry).unwrap();

                if !ctrl.supported() {
                    println!("Unsupported APLIC");
                    break;
                }

                println!("Found supported {:#x?}", ctrl);
            },
            EntryType::Plic => {
                let ctrl = madt::Plic::from_entry(entry).unwrap();

                if !ctrl.supported() {
                    println!("Unsupported PLIC");
                    break;
                }

                println!("Found supported {:#x?}", ctrl);
            },
            etype => panic!("Unhandled etype {:?}", etype)
        }
    }*/

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

    gent_kern::scheduler::spawn_kernel_thread(gent_kern::dev::window::display_thread, 4);
    gent_kern::scheduler::spawn_kernel_thread(draw, 4);

    let timer = gent_kern::arch::timer::get_timer();
    gent_kern::arch::timer::set_timer(timer);
    slow_loop();
}

fn draw() -> ! {
    use gent_kern::dev::window;
    use core::num::NonZeroUsize;

    let mut win_id_raw = 0;
    let mut buffer = core::ptr::null_mut();
    let mut complete = false;
    const SIZE: usize = 256;
    window::DISPLAY_QUEUE.push(
        window::Request::AddWindow(SIZE, SIZE, &mut win_id_raw, &mut buffer, &mut complete)
    );

    while unsafe { !(&complete as *const bool).read_volatile() } {}

    let win_id = NonZeroUsize::new(win_id_raw).unwrap();

    for i in 0..SIZE * SIZE {
        unsafe {
            buffer.add(i).write_volatile(0xffffff00);
        }
    }

    println!("Finished setting up yellow window {} buffer {:?}", win_id, buffer);

    let mut win_id_raw = 0;
    let mut buffer2 = core::ptr::null_mut();
    let mut complete = false;
    window::DISPLAY_QUEUE.push(
        window::Request::AddWindow(128, 128, &mut win_id_raw, &mut buffer2, &mut complete)
    );

    while unsafe { !(&complete as *const bool).read_volatile() } {}

    let win_id2 = NonZeroUsize::new(win_id_raw).unwrap();

    for i in 0..128 * 128 {
        unsafe {
            buffer2.add(i).write_volatile(0xffffffff);
        }
    }

    println!("Finished setting up white window {} buffer {:?}", win_id2, buffer2);

    for i in 0.. {
        window::DISPLAY_QUEUE.push(
            window::Request::Move(win_id2, i, i)
        );
    }

    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {:#?}", info);
    loop {
        gent_kern::arch::utils::slow();
    }
}

fn slow_loop() -> ! {
    loop {
        gent_kern::arch::utils::slow();
    }
}