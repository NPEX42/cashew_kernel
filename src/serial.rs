use core::fmt::{Arguments, Write};
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

const COMM0_ADDR: u16 = 0x3F8;

lazy_static! {
    static ref COMM0: Mutex<SerialPort> = unsafe { Mutex::new(SerialPort::new(COMM0_ADDR)) };
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        COMM0
            .lock()
            .write_fmt(args)
            .expect("Failed To Print To Serial 0");
    });
}


pub fn write_u8(byte: u8) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        COMM0
            .lock()
            .send(byte)
    });
}

pub fn read_u8() -> Option<u8> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        Some(
            COMM0
            .lock()
            .receive()
        )
    })
}

/// Print To COMM 0
#[macro_export]
macro_rules! sprint {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    }
}
