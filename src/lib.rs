#![no_std]
#![feature(once_cell)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(derive_default_enum)]
#![feature(const_btree_new)]
#![feature(const_fn_trait_bound)]
#![feature(int_log)]

use core::panic::PanicInfo;

pub mod arch;
pub mod ata;
pub mod csh;
pub mod data;
pub mod device;
pub mod fonts;
pub mod fuse;
pub mod graphics_2d;
pub mod input;
pub mod locked;
pub mod logger;
pub mod mem;
pub mod pit;
pub mod serial;
pub mod terminal;
pub mod vga;
pub mod vfs;

extern crate alloc;

pub use alloc::*;

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

#[alloc_error_handler]
fn alloc_error(layout: alloc::alloc::Layout) -> ! {
    panic!(
        "Allocation Error: Unable To Allocate {} Bytes (Align: {})",
        layout.size(),
        layout.align()
    );
}
