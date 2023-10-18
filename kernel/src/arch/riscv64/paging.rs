use crate::println;

pub fn get_root_table() -> RootTable {
    let satp_raw: usize;
    unsafe {
        core::arch::asm!(
            "csrr {satp}, satp",
            satp = out(reg) satp_raw,
        );
    }

    let satp: super::csr::Satp = unsafe {core::mem::transmute(satp_raw)};

    RootTable(satp.phys().to_virt().to_mut_ptr(), satp.mode())
}

pub enum Mode {
    Bare = 0,
    Sv39 = 8,
    Sv48 = 9,
    Sv57 = 10,
    Sv64 = 11,
}

impl Mode {
    pub fn to_level(&self) -> usize {
        use Mode::*;

        match self {
            Bare => 0,
            Sv39 => 3,
            Sv48 => 4,
            Sv57 => 5,
            Sv64 => panic!("Sv64 is not valid")
        }
    }

    pub fn max_size(&self) -> PageSize {
        use Mode::*;

        match self {
            Bare => PageSize::None,
            Sv39 => PageSize::Kilopage,
            Sv48 => PageSize::Terapage,
            Sv57 => PageSize::Petapage,
            Sv64 => panic!("Sv64 is undefined as of implementation"),
        }
    }

    pub fn higher_half(&self) -> usize {
        use Mode::*;

        match self {
            Bare => 0,
            Sv39 => 0xffffffc000000000,
            Sv48 => 0xffff800000000000,
            Sv57 => 0xff00000000000000,
            Sv64 => panic!("Sv64 is undefined as of implementation")
        }
    }
}

#[repr(u64)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PageSize {
    Petapage = 0x1000_0000_0000,
    Terapage = 0x80_0000_0000,
    Gigapage = 0x4000_0000,
    Megapage = 0x20_0000,
    Kilopage = 0x1000,
    None =     0x0,
}

impl PageSize {
    pub fn from_level(level: usize) -> Self {
        match level {
            0 => Self::None,
            1 => Self::Kilopage,
            2 => Self::Megapage,
            3 => Self::Gigapage,
            4 => Self::Terapage,
            5 => Self::Petapage,
            _ => panic!("Invalid page level: {}", level)
        }
    }

    pub fn to_level(&self) -> usize {
        use PageSize::*;

        match self {
            Kilopage => 1,
            Megapage => 2,
            Gigapage => 3,
            Terapage => 4,
            Petapage => 5,
            None => 0,
        }
    }

    pub fn from_size(size: usize) -> Self {
        match size {
            0 => Self::None,
            0x1000 => Self::Kilopage,
            0x200000 => Self::Megapage,
            0x40000000 => Self::Gigapage,
            0x8000000000 => Self::Terapage,
            0x100000000000 => Self::Petapage,
            _ => panic!("Unkown page size {}", size)
        }
    }

    /// Takes a size and if its not an exact size, it rounds it up a size
    pub fn from_size_ceil(size: usize) -> Self {
        match size {
            0 => Self::None,
            0x1..=0x1000 => Self::Kilopage,
            0x1001..=0x200000 => Self::Megapage,
            0x200001..=0x40000000 => Self::Gigapage,
            0x40000001..=0x8000000000 => Self::Terapage,
            0x8000000001..=0x100000000000 => Self::Petapage,
            _ => panic!("Unkown page size {}", size)
        }
    }
}

#[derive(Clone, Copy)]
pub struct PagePermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub user: bool,
    pub global: bool,
    pub dealloc: bool,
}

#[derive(Debug)]
pub enum PageError {
    MappingExists(PageTableEntry),
    NoMapping,
    UnmappingSizeMismatch,
    InvalidSize,
}

pub struct RootTable(*mut PageTable, Mode);

impl RootTable {
    /// # Safety
    /// Can change what memory addresses are valid to access, and how its valid to access it.
    pub unsafe fn map(
        &mut self, 
        vaddr: crate::mem::VirtualAddress, 
        paddr: crate::mem::PhysicalAddress, 
        perms: PagePermissions, 
        size: PageSize
    ) -> Result<(), PageError> {
        let mut cur_level = self.1.to_level();
        let mut table = self.0;

        if size > self.1.max_size() {
            return Err(PageError::InvalidSize);
        } else if size == PageSize::None {
            // If the page size is zero, why are you trying to map?
            return Ok(());
        }

        while cur_level >= size.to_level() {
            let entry = &mut (*table)[vaddr.vpn(cur_level)];

            match entry.entry() {
                Entry::Table(next_table) => {
                    table = next_table.cast_mut();
                },
                Entry::Page(_page) => {
                    return Err(PageError::MappingExists(*entry));
                },
                Entry::Invalid => {
                    if cur_level != size.to_level() {
                        let table_paddr = crate::mem::PHYS.alloc(0x1000, vmem::AllocStrategy::NextFit).unwrap();
                        let table_paddr = crate::mem::PhysicalAddress::new(table_paddr);

                        entry.0 = 0;

                        entry.set_phys(table_paddr);
                        entry.set_valid(true);

                        table = table_paddr.to_virt().to_mut_ptr();
                    } else {
                        entry.0 = 0;
                        entry.set_perms(perms);
                        entry.set_phys(paddr);
            
                        entry.set_valid(true);
            
                        return Ok(());
                    }
                }
            }

            cur_level -= 1;
        }
        
        unreachable!()
    }

    /// # Safety
    /// Can change what memory addresses are valid to access, and how its valid to access it.
    pub unsafe fn unmap(
        &mut self, 
        vaddr: crate::mem::VirtualAddress, 
        size: PageSize
    ) -> Result<(), PageError> {
        let mut cur_level = self.1.to_level();
        let mut table = self.0;

        if size > self.1.max_size() {
            return Err(PageError::InvalidSize);
        } else if size == PageSize::None {
            // If the page size is zero, why are you trying to unmap?
            return Ok(());
        }

        while cur_level >= size.to_level() {
            let entry = &mut (*table)[vaddr.vpn(cur_level)];

            match entry.entry() {
                Entry::Table(next_table) => {
                    if cur_level != size.to_level() {
                        table = next_table.cast_mut();
                    } else {
                        todo!("Recursively unmap a table")
                    }
                },
                Entry::Page(page) => {
                    if cur_level == size.to_level() {
                        let hhdm_addr = page as usize;
                        let paddr = hhdm_addr - crate::mem::HHDM_OFFSET.load(core::sync::atomic::Ordering::Relaxed);
                        if entry.dealloc() {
                            crate::mem::PHYS.free(paddr, size as usize);
                        }
                        return Ok(());
                    } else {
                        return Err(PageError::UnmappingSizeMismatch);
                    }
                },
                Entry::Invalid => {
                    return Err(PageError::NoMapping);
                }
            }

            cur_level -= 1;
        }
        
        unreachable!()
    }

    pub fn get_entry(&self, vaddr: crate::mem::VirtualAddress) -> &'static mut PageTableEntry {
        // `self.1` contains the paging mode
        // `self.0` contains a mutable pointer to the page tables, we cast it to constant for safety reasons
        let mut cur_level = self.1.to_level();
        let mut table = self.0;

        loop {
            unsafe {
                // We get the current entry by dereferencing the table, and indexing it based on the virtual address' vpn for the current level
                let entry = &mut (*table)[vaddr.vpn(cur_level)];
                
                // `entry.entry()` returns an `entry` type, which is an enum identifying an entry as a table, page, or invalid entry
                match entry.entry() {
                    Entry::Table(next_table) => {
                        table = next_table.cast_mut();
                    },
                    _ => {
                        return entry
                    }
                }

                // If the current level is 0 before we decrement the level again, then we found a table entry way lower than we should have
                // Will change if I encounter it
                if cur_level == 0 {
                    panic!("Uhhh :clueless:");
                }

                cur_level -= 1;
            }
        }
    }

    /// Finds the lowest entry in that virtual address, returns page level, and entry
    pub fn read(&self, vaddr: crate::mem::VirtualAddress) -> (Entry, usize) {
        // `self.1` contains the paging mode
        // `self.0` contains a mutable pointer to the page tables, we cast it to constant for safety reasons
        let mut cur_level = self.1.to_level();
        let mut table = self.0.cast_const();

        loop {
            unsafe {
                // We get the current entry by dereferencing the table, and indexing it based on the virtual address' vpn for the current level
                let entry = (*table)[vaddr.vpn(cur_level)];
                
                // `entry.entry()` returns an `entry` type, which is an enum identifying an entry as a table, page, or invalid entry
                match entry.entry() {
                    Entry::Table(next_table) => {
                        table = next_table;
                    },
                    _ => {
                        return (entry.entry(), cur_level)
                    }
                }

                // If the current level is 0 before we decrement the level again, then we found a table entry way lower than we should have
                // Will change if I encounter it
                if cur_level == 0 {
                    panic!("Uhhh :clueless:");
                }

                cur_level -= 1;
            }
        }
    }

    pub fn swap(
        &mut self, 
        vaddr: crate::mem::VirtualAddress,
        procid: u128,
    ) -> Result<(), PageError> {
        let entry = self.read(vaddr);
        let size = PageSize::from_level(entry.1);

        let entry = self.get_entry(vaddr);
        entry.set_valid(false);

        if !entry.read() {
            return Err(PageError::NoMapping);
        } 

        let result = if entry.swapped() {
            // Swap it back in
            let phys = crate::mem::PHYS.alloc(size as usize, vmem::AllocStrategy::NextFit).unwrap();

            let disk_info = if vaddr.is_kern() {
                let mut lock = crate::mem::swap::KERN_SWAP.lock();
                lock.remove(&vaddr).unwrap()
            } else {
                let mut lock = crate::mem::swap::SWAP_MAN.lock();
                lock.remove(&(procid, vaddr)).unwrap()
            };

            let disk_id = disk_info.0;
            let block = disk_info.1;

            let lock = crate::dev::blockdev::PARTS.lock();
            let disk = lock.get(&disk_id).unwrap();
            println!("Getting disks");

            let buf = unsafe {core::slice::from_raw_parts_mut(
                (phys + crate::mem::HHDM_OFFSET.load(core::sync::atomic::Ordering::Relaxed)) as *mut u8, 
                size as usize
            )};

            disk.read(buf, block).unwrap();

            entry.set_swapped(false);
            entry.set_ppn((phys >> 12) as u64);
            entry.set_valid(true);
            Ok(())
        } else {
            // Swap it back out
            entry.set_swapped(true);

            let mut lock = crate::dev::blockdev::PARTS.lock();
            let part = lock.get_mut(&0).unwrap();
    
            let block_base = part.alloc_blocks((size as usize).div_ceil(part.blocksize()));

            if vaddr.is_kern() {
                let mut swaplock = crate::mem::swap::KERN_SWAP.lock();
                swaplock.insert(vaddr, (0, block_base));
            } else {
                let mut swaplock = crate::mem::swap::SWAP_MAN.lock();
                swaplock.insert((procid, vaddr), (0, block_base));
            }

            let buf = unsafe {core::slice::from_raw_parts(
                entry.phys().to_virt().to_ptr(), 
                size as usize
            )};

            part.write(buf, block_base).unwrap();
            Ok(())
        };

        sfence();
        result
    }
}

pub fn sfence() {
    unsafe {
        core::arch::asm!("sfence.vma")
    }
}

#[repr(transparent)]
pub struct PageTable([PageTableEntry; 512]);

impl core::ops::Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl core::ops::IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

bitfield::bitfield! {
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    pub struct PageTableEntry(u64);
    impl Debug;

    u64;
    pub valid, set_valid: 0;
    read, set_read: 1;
    write, set_write: 2;
    exec, set_exec: 3;
    user, set_user: 4;
    global, set_global: 5;
    accessed, set_accessed: 6;
    dirty, set_dirty: 7;
    dealloc, set_dealloc: 8;
    pub swapped, set_swapped: 9;
    ppn, set_ppn: 53, 10;
    reserved, set_reserved: 60, 54;
    pbmt, set_pbmt: 62, 61;
    n, set_n: 63;
}

impl PageTableEntry {
    pub fn set_perms(&mut self, perms: PagePermissions) {
        self.set_read(perms.read);
        self.set_write(perms.write);
        self.set_exec(perms.execute);
        self.set_user(perms.user);
        self.set_global(perms.global);
        self.set_dealloc(perms.dealloc);
    }

    pub fn is_read(&self) -> bool {
        self.read()
    }

    pub fn is_write(&self) -> bool {
        self.write()
    }

    pub fn is_exec(&self) -> bool {
        self.exec()
    }
}

pub enum Entry {
    Page(*const u8),
    Table(*const PageTable),
    Invalid,
}

impl PageTableEntry {
    pub fn set_phys(&mut self, val: crate::mem::PhysicalAddress) {
        self.set_ppn((val.addr() >> 12) as u64);
    }

    pub fn phys(&self) -> crate::mem::PhysicalAddress {
        crate::mem::PhysicalAddress::new((self.ppn() << 12) as usize)
    }

    pub fn entry(&self) -> Entry {
        let phys = self.phys();

        if self.valid() {
            let addr = phys.addr() + crate::mem::HHDM_OFFSET.load(core::sync::atomic::Ordering::Relaxed);

            if self.read() | self.write() | self.exec() {
                Entry::Page(addr as *const u8)
            } else {
                Entry::Table(addr as *const PageTable)
            }
        } else {
            Entry::Invalid
        }
    }
}