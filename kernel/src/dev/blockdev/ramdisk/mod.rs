// The RAMDisk driver allows access to multiple in-RAM 'disks'
// This can be useful for testing out designs without actually messing with the disks of a system.

use crate::println;

pub struct RamDisk {
    buffer: &'static mut [u8],
    blocksize: usize,
    blocks: usize,
    block_map: vmem::Vmem<'static, 'static>
}

impl RamDisk {
    pub fn new(blocks: usize, blocksize: usize) -> Self {
        let totalsize = blocksize * blocks;

        let alloc = unsafe {
            alloc::alloc::alloc(
                alloc::alloc::Layout::from_size_align(
                    totalsize, 
                    blocksize 
                ).unwrap()
            )
        };

        let block_map = vmem::Vmem::new(
            alloc::borrow::Cow::Borrowed("RAMDISK"), 
            blocksize, 
            None
        );

        block_map.add(0, totalsize).unwrap();

        Self { 
            buffer: unsafe {core::slice::from_raw_parts_mut(alloc, totalsize)}, 
            blocksize,
            blocks,
            block_map
        }
    }
}

impl super::Disk for RamDisk {
    fn blocks(&self) -> usize {
        self.blocks
    }

    fn blocksize(&self) -> usize {
        self.blocksize
    }

    fn read(&self, buffer: &mut [u8], block: usize) {
        assert!(block < self.blocks, "attempt to read too many blocks");

        let base = block * self.blocksize;

        for (index, byte) in buffer.iter_mut().enumerate() {
            let selfidx = base + index;

            *byte = self.buffer[selfidx];
        }
    }

    fn write(&mut self, data: &[u8], block: usize) {
        assert!(block < self.blocks, "attempt to read too many blocks");

        let base = block * self.blocksize;

        for (index, byte) in data.iter().enumerate() {
            let selfidx = base + index;

            self.buffer[selfidx] = *byte;
        }
    }

    fn alloc_blocks(&self, blocks: usize) -> usize {
        println!("Allocating {} blocks", blocks);
        self.block_map.alloc(blocks * self.blocksize, vmem::AllocStrategy::NextFit).unwrap()
    }
}