use alloc::{collections::BTreeMap, vec::Vec};
use spin::Mutex;

use super::VirtualAddress;

/// Key: process ID, Virtual address.
/// Value: Partition ID, Block
pub static SWAP_MAN: Mutex<BTreeMap<(u128, VirtualAddress), (usize, usize)>> = Mutex::new(BTreeMap::new());

/// Key: Virtual address.
/// Value: Partition ID, Block
pub static KERN_SWAP: Mutex<BTreeMap<VirtualAddress, (usize, usize)>> = Mutex::new(BTreeMap::new());

pub static SWAP_PARTS: Mutex<Vec<usize>> = Mutex::new(Vec::new());

pub static SWAP_LOC: vmem::Vmem = vmem::Vmem::new(alloc::borrow::Cow::Borrowed("SWAP_LOCATIONS"), 4096, None);

pub fn init_swap() {
}