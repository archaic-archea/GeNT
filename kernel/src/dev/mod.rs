pub mod fw_cfg;
pub mod ramfb;
pub mod blockdev;
pub mod virtio;
pub mod aplic;
pub mod uart;
pub mod window;

linkset::declare!(pub DRIVERS: DriverEntry);

pub struct DriverEntry {
    pub id: &'static str,
    pub init: fn(node: lai::Node),
}

unsafe impl Send for DriverEntry {}
unsafe impl Sync for DriverEntry {}