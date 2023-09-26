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
}

#[repr(C)]
#[derive(Debug)]
pub struct MadtEntry {
    etype: EntryType,
    len: u8,
}

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum EntryType {
    RiscvIntController = 0x18,
    Imsic = 0x19,
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
pub struct ImsicTable {
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