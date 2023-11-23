pub static UART: AtomicPtr<Uart16550> = AtomicPtr::new(core::ptr::null_mut());

use crate::println;

use super::DRIVERS;

linkset::entry!(DRIVERS, super::DriverEntry, super::DriverEntry {
    id: "PNP0501",
    init,
});

fn init(node: lai::Node) {
    let mmio = node.child("_CRS").unwrap().eval().unwrap();
    let mmio = mmio.get_buffer().unwrap();

    assert!(mmio[0] == 0b10000110);
    assert!(mmio[1] == 0x09);
    assert!(mmio[2] == 0x00);

    let base: *mut Uart16550 = crate::mem::PhysicalAddress::new(
        u32::from_le_bytes([mmio[4], mmio[5], mmio[6], mmio[7]]) as usize
    ).to_virt().to_mut_ptr();

    UART.store(base, Ordering::Relaxed);

    *crate::PRINTFN.lock() = _print;
}

#[repr(C)]
pub struct Uart16550 {
    data_register: u8
}

use core::{fmt, sync::atomic::{AtomicPtr, Ordering}};

impl fmt::Write for Uart16550 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for character in s.chars() {
            self.data_register = character as u8;
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write; 
    unsafe {
        (*UART.load(Ordering::Relaxed)).write_fmt(args).unwrap();
    }
}