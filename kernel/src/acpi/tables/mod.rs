pub mod madt;

#[repr(C)]
pub struct Rsdp {
    sig: [u8; 8],
    checksum: u8,
    oemid: [u8; 6],
    rev: u8,
    rsdt_addr: u32,
    len: u32,
    xsdt_addr: u64,
    ext_checksum: u8,
    _res: [u8; 3]
}

impl Rsdp {
    pub fn rsdt(&self) -> &'static Rsdt {
        let ptr = if self.rev >= 1 {
            self.xsdt_addr as *mut Rsdt
        } else {
            self.rsdt_addr as *mut Rsdt
        };

        unsafe {
            assert!(!ptr.is_null());
            &*ptr
        }
    }

    pub fn sig(&self) -> &str {
        unsafe {
            let slice = core::slice::from_raw_parts(core::ptr::addr_of!(self.sig) as *const u8, 8);
            let str = core::str::from_utf8(slice).unwrap();

            str
        }
    }

    pub fn oemid(&self) -> &str {
        unsafe {
            let slice = core::slice::from_raw_parts(core::ptr::addr_of!(self.oemid) as *const u8, 6);
            let str = core::str::from_utf8(slice).unwrap();

            str
        }
    }
}

pub struct Rsdt {
    header: SdtHeader,
}

impl Rsdt {
    pub fn name(&self) -> &str {
        self.header.name()
    }
    
    pub fn header_ptr(&self, index: usize) -> *const SdtHeader {
        unsafe {
            let ptr = core::ptr::addr_of!(*self).add(1);

            if self.header.rev >= 1 {
                let ptr = ptr as *const u64;
                assert!(*ptr != 0);
    
                *ptr.add(index) as *const SdtHeader
            } else {
                let ptr = ptr as *const u32;
                assert!(*ptr != 0);

                *ptr.add(index) as *const SdtHeader
            }
        }
    } 

    pub(crate) fn from_ptr(ptr: *mut u8) -> &'static Self {
        unsafe {
            &*(ptr as *mut _)
        }
    }

    pub fn entries(&self) -> usize {
        let ptr_size = if self.header.rev >= 1 {
            8
        } else {
            4
        };

        (self.header.len as usize - core::mem::size_of::<SdtHeader>()) / ptr_size
    }

    pub fn get_table(&self, sig: &str) -> *const SdtHeader {

        let mut subtable = false;
        let mut subtable_sig = "";

        let sig = if sig == "DSDT" {
            subtable = true;
            subtable_sig = "DSDT";
            "FACP"
        } else {
            sig
        };

        let mut main_table = core::ptr::null();

        for i in 0..self.entries() {
            let table = unsafe {&*self.header_ptr(i)};
            
            if table.name() == sig {
                main_table = self.header_ptr(i);
                break;
            }
        }
        
        if subtable {
            if sig == "FACP" {
                let table = unsafe {&*(main_table as *const Fadt)};

                main_table = table.get_table(subtable_sig);
            } else {
                panic!("Unhandled signature {:#?}", sig);
            }
        }

        main_table
    }
}

impl IntoIterator for &'static Rsdt {
    type IntoIter = SdtIter;
    type Item = &'static SdtHeader;

    fn into_iter(self) -> Self::IntoIter {
        SdtIter(self, 0)
    }
}

pub struct SdtIter(&'static Rsdt, usize);

impl Iterator for SdtIter {
    type Item = &'static SdtHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 >= self.0.entries() {
            return None;
        }

        let entry = self.0.header_ptr(self.1);
        assert!(!entry.is_null(), "Error occured {:?}", entry);
        self.1 += 1;

        unsafe {
            Some(&*entry)
        }
    }
}

#[allow(unused)]
#[repr(C)]
pub struct SdtHeader {
    pub sig: [u8; 4],
    len: u32,
    rev: u8,
    _checksum: u8,
    oemid: [u8; 6],
    oemtableid: [u8; 8],
    oemrev: u32,
    creator_id: u32,
    creatorrev: u32
}

impl SdtHeader {
    pub fn name(&self) -> &str {
        unsafe {
            let slice = core::slice::from_raw_parts(core::ptr::addr_of!(self.sig) as *const u8, 4);
            let str = core::str::from_utf8(slice).unwrap();

            str
        }
    }
}

#[repr(C)]
pub struct Fadt {
    header: SdtHeader,
    firmwarectl: u32,
    dsdt: u32,
 
    // field used in ACPI 1.0; no longer in use, for compatibility only
    _res: u8,
 
    power_mgr_profile: u8,
    sci_int: u16,
    smi_cmd_port: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4bios_req: u8,
    pstate_ctrl: u8,
    pm1a_event_blk: u32,
    pm1b_event_blk: u32,
    pm1a_ctrl_blk: u32,
    pm1b_ctrl_blk: u32,
    pm2_ctrl_blk: u32,
    pm_timer_blk: u32,
    gpe0_blk: u32,
    gpe1_blk: u32,
    pm1_event_len: u8,
    pm1_ctrl_len: u8,
    pm2_ctrl_len: u8,
    pm_timer_len: u8,
    gpe0_len: u8,
    gpe1_len: u8,
    gpe1_base: u8,
    cstate_ctrl: u8,
    worst_c2_latency: u16,
    worst_c3_latency: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alarm: u8,
    month_alarm: u8,
    cent_alarm: u8,
 
    // reserved in ACPI 1.0; used since ACPI 2.0+
    boot_arch_flags: u16,
 
    _res2: u8,
    flags: u32,
 
    // 12 byte structure; see below for details
    reset_reg: AddrStruct,
    
    reset_val: u8,
    _res3: [u8; 3],
 
    // 64bit pointers - Available on ACPI 2.0+
    x_firmware_ctrl: u64,
    x_dsdt: u64,

    x_pm1a_event_blk: AddrStruct,
    x_pm1b_event_blk: AddrStruct,
    x_pm1a_ctrl_blk: AddrStruct,
    x_pm1b_ctrl_blk: AddrStruct,
    x_pm2_ctrl_blk: AddrStruct,
    x_pm_timer_blk: AddrStruct,
    x_gpe0_blk: AddrStruct,
    x_gpe1_blk: AddrStruct,
}

#[allow(unused)]
#[repr(packed)]
pub struct AddrStruct {
    addr_space: u8,
    bit_width: u8,
    bit_offset: u8,
    access_size: u8,
    addr: u64,
}

impl Fadt {
    pub fn get_table(&self, sig: &str) -> *const SdtHeader {
        match sig {
            "DSDT" => self.dsdt as *const SdtHeader,
            _ => panic!("Unrecognized signature {:#?}", sig)
        }
    }
}