#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

pub mod global;

#[derive(Clone, Copy)]
pub enum Mode {
    Supervisor,
    User,
}

#[macro_export]
macro_rules! no_io_ports {
    ($t:ident) => {
        impl $crate::arch::global::PortAccess for $t {
            unsafe fn read(_location: usize) -> $t {
                panic!("No IO Ports!")
            }
            unsafe fn write(_location: usize, _val: $t) {
                panic!("No IO Ports!")
            }
        }
    };
}