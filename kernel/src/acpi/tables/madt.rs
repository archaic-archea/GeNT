#[repr(C)]
pub struct Madt {
    header: super::SdtHeader,
    lapic_addr: u32,
    flags: u32,
}

impl Madt {
    pub fn first_entry(&self) -> *const MadtEntry {
        unsafe {
            let ptr = core::ptr::addr_of!(*self).add(1);
            ptr as *const MadtEntry
        }
    }

    pub fn entry(&self, index: usize) -> Option<&'static MadtEntry> {
        unsafe {
            let mut ptr = self.first_entry();
            let mut total_len = core::mem::size_of::<Madt>() + (*ptr).len as usize;
            
            for _ in 0..index {
                if total_len >= self.header.len as usize {
                    return None;
                }
                
                let len = (*ptr).len;
                ptr = ptr.byte_add(len as usize);
                total_len += len as usize;
            }

            Some(&*ptr)
        }
    }

    pub fn iter(&self) -> IterMadt {
        IterMadt { madt: self, cur: 0 }
    }
}

pub struct IterMadt {
    madt: *const Madt,
    cur: usize,
}

impl Iterator for IterMadt {
    type Item = &'static MadtEntry;
    fn next(&mut self) -> Option<Self::Item> {
        self.cur += 1;
        unsafe {(*self.madt).entry(self.cur - 1)}
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct MadtEntry {
    etype: EntryType,
    len: u8,
}

impl MadtEntry {
    pub fn etype(&self) -> EntryType {
        self.etype
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EntryType {
    BIOPic = 0x16,
    LPCPic = 0x17,
    RiscvIntController = 0x18,
    Imsic = 0x19,
    Aplic = 0x1A,
    Plic = 0x1B,
}

#[repr(C)]
#[derive(Debug)]
pub struct Aplic {
    entry: MadtEntry,
    version: u8,
    aplic_id: u8,
    flags: u32,
    hard_id: [core::ffi::c_char; 8],
    idcs: u16,
    ext_ints: u16,
    int_base: u32,
    aplic_addr: u64,
    aplic_size: u32,
}

impl Aplic {
    pub fn from_entry(entry: &MadtEntry) -> Option<&Self> {
        if entry.etype != EntryType::Aplic {
            return None;
        }

        unsafe {
            let ptr = core::ptr::addr_of!(*entry) as *const Self;

            Some(&*ptr)
        }
    }

    pub fn supported(&self) -> bool {
        let mut supported = self.version == 1;
        supported |= self.entry.len == 36;

        supported
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Plic {
    entry: MadtEntry,
    version: u8,
    plic_id: u8,
    hard_id: [core::ffi::c_char; 8],
    ext_int: u16,
    max_priority: u16,
    flags: u32,
    plic_size: u32,
    plic_addr: u64,
    int_base: u32,
}

impl Plic {
    pub fn from_entry(entry: &MadtEntry) -> Option<&Self> {
        if entry.etype != EntryType::Plic {
            return None;
        }

        unsafe {
            let ptr = core::ptr::addr_of!(*entry) as *const Self;

            Some(&*ptr)
        }
    }

    pub fn supported(&self) -> bool {
        let mut supported = self.version == 1;
        supported |= self.entry.len == 36;

        supported
    }
}

/// RINTC structure as of revision 1
#[repr(C)]
#[derive(Debug)]
pub struct RiscvIntController {
    entry: MadtEntry,
    version: u8,
    _res: u8,
    flags: u32,
    hartid: u64,
    acpi_proc_id: u32,
    ext_int_id: ExtIntID,
    /// Ignore if IMSIC is not present
    imsic_addr: u64,
    /// Ignore if IMSIC is not present
    imsic_size: u32
}

impl RiscvIntController {
    pub fn from_entry(entry: &MadtEntry) -> Option<&Self> {
        if entry.etype != EntryType::RiscvIntController {
            return None;
        }

        unsafe {
            let ptr = core::ptr::addr_of!(*entry) as *const Self;

            Some(&*ptr)
        }
    }

    pub fn supported(&self) -> bool {
        let mut supported = self._res == 0;
        supported |= self.version == 1;
        supported |= self.entry.len == 36;

        supported
    }
}

bitfield::bitfield! {
    #[repr(transparent)]
    pub struct ExtIntID(u32);
    impl Debug;
    idc_id, _: 15, 0;
    _res, _: 23, 16;
    plic_id, _: 31, 24;
}

#[repr(C)]
#[derive(Debug)]
pub struct Imsic {
    entry: MadtEntry,
    version: u8,
    _res: u8,
    flags: u32,
    smode_int_ids: u16,
    guest_int_ids: u16,
    guest_idx: u8,
    hart_idx: u8,
    group_idx: u8,
    group_idx_shift: u8,
}

impl Imsic {
    pub fn from_entry(entry: &MadtEntry) -> Option<&Self> {
        if entry.etype != EntryType::Imsic {
            return None;
        }

        unsafe {
            let ptr = core::ptr::addr_of!(*entry) as *const Self;

            Some(&*ptr)
        }
    }

    pub fn supported(&self) -> bool {
        let mut supported = self._res == 0;
        supported |= self.version == 1;
        supported |= self.entry.len == 16;

        supported
    }
}