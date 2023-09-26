pub static UART: AtomicPtr<Uart16550> = AtomicPtr::new(0x1000_0000 as *mut Uart16550);

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

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn flush(&self) {
    }

    fn log(&self, record: &log::Record) {
        use crate::println;
        println!("[{}]{}: {}", record.level(), record.target(), record.args());
    }
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!{"{}\n", format_args!($($arg)*)});
}
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write; 
    unsafe {
        (*UART.load(Ordering::Relaxed)).write_fmt(args).unwrap();
    }
}