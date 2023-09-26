use alloc::collections::BTreeMap;
use spin::Mutex;

mod ramdisk;

pub static DISKS: Mutex<BTreeMap<usize, &'static mut dyn Disk>> = Mutex::new(BTreeMap::new());

pub trait Disk: Send + Sync {
    fn blocksize(&self) -> usize;
    fn blocks(&self) -> usize;
    fn write(&mut self, data: &[u8], block: usize);
    fn read(&self, buffer: &mut [u8], block: usize);
    fn alloc_blocks(&self, blocks: usize) -> usize;
}

pub fn init() {
    let ramdisk = alloc::boxed::Box::new(ramdisk::RamDisk::new(16, 1024));
    DISKS.lock().insert(0, alloc::boxed::Box::leak(ramdisk));
}