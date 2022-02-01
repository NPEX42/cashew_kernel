#![no_std]
#![feature(once_cell)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

use core::panic::PanicInfo;

pub mod vga;
pub mod logger;
pub mod locked;
pub mod serial;
pub mod fonts;
pub mod terminal;
pub mod colors;
pub mod pit;
pub mod graphics_2d;
pub mod input;
pub mod data;
pub mod arch;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {

    sprint!("Panic: {}\n", info);
    //kerr!("== Kernel Panic ==\n{}", info);
    loop {}
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}