use alloc::collections::BTreeMap;
use spin::Mutex;

mod ramdisk;
mod gpt;

static DISKS: Mutex<BTreeMap<usize, &'static mut dyn Disk>> = Mutex::new(BTreeMap::new());
pub static PARTS: Mutex<BTreeMap<usize, &'static mut Partition>> = Mutex::new(BTreeMap::new());

pub trait Disk: Send + Sync {
    fn blocksize(&self) -> usize;
    fn blocks(&self) -> usize;
    fn chs_end(&self) -> u32;
    fn write(&mut self, data: &[u8], block: usize);
    fn read(&self, buffer: &mut [u8], block: usize);
}

#[derive(Debug)]
pub enum DiskError {
    InvalidBlock,
    PartitionRangeError,
}

pub fn init() {
    let ramdisk = alloc::boxed::Box::new(ramdisk::RamDisk::new(16, 1024));
    let ramdisk = alloc::boxed::Box::leak(ramdisk);

    gpt::init_disk_gpt(ramdisk);

    let swap_part = alloc::boxed::Box::new(
        Partition::new(
            0, 
            4, 
            ramdisk.blocks(),
            ramdisk.blocksize()
        )
    );
    let partition = alloc::boxed::Box::leak(swap_part);

    DISKS.lock().insert(0, ramdisk);
    PARTS.lock().insert(0, partition);

    crate::mem::swap::SWAP_PARTS.lock().push(0);
}

pub struct Partition {
    disk_id: usize,
    blocksize: usize,
    blocks: usize,
    block_map: vmem::Vmem<'static, 'static>
}

impl Partition {
    pub fn new(disk_id: usize, block_base: usize, blocks: usize, blocksize: usize) -> Self {
        let block_map = vmem::Vmem::new(
            alloc::borrow::Cow::Borrowed("RAMDISK"), 
            1, 
            None
        );

        block_map.add(block_base, blocks).unwrap();

        Self { 
            disk_id,
            blocksize,
            blocks,
            block_map
        }
    }
    
    pub fn diskid(&self) -> usize {
        self.disk_id
    }

    pub fn blocksize(&self) -> usize {
        self.blocksize
    }

    pub fn blocks(&self) -> usize {
        self.blocks
    }

    pub fn write(&mut self, data: &[u8], block: usize) -> Result<(), DiskError> {
        let mut diskslock = DISKS.lock();
        let disk = diskslock.get_mut(&self.disk_id).unwrap();

        disk.write(data, block);

        Ok(())
    }

    pub fn read(&self, data: &mut [u8], block: usize) -> Result<(), DiskError> {
        let mut diskslock = DISKS.lock();
        let disk = diskslock.get_mut(&self.disk_id).unwrap();

        disk.read(data, block);

        Ok(())
    }
    
    pub fn alloc_blocks(&self, blocks: usize) -> usize {
        self.block_map.alloc(blocks, vmem::AllocStrategy::NextFit).unwrap()
    }
}