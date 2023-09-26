use alloc::collections::BTreeMap;
use spin::Mutex;

use super::VirtualAddress;

/// Key: process ID (0 refers to kernel), Virtual address.
/// Value: Disk ID, Block
pub static SWAP_MAN: Mutex<BTreeMap<(u128, VirtualAddress), (usize, usize)>> = Mutex::new(BTreeMap::new());
