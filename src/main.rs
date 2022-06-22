#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![test_runner(cashew_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use core::arch::asm;




use alloc::{vec::Vec};
#[cfg(not(test))]
use bootloader::entry_point;
use bootloader::BootInfo;
use cashew_kernel::{ata, graphics_2d::*, kerr, println, csh, net::{self}};


#[cfg(not(test))]
entry_point!(kernel_main);

static mut FRAME: Frame = Frame::new();

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(mut fb) = boot_info.framebuffer.as_mut() {
        cashew_kernel::boot(boot_info);


        println!("Booting Complete");

        

        net::init();


        ata::cache_stats();
        csh::main(Vec::new());
        cashew_kernel::shutdown();
    } else {
        kerr!("Failed To Find Framebuffer, Please File An Issue On Github.\n");
    }
    loop {
        cashew_kernel::arch::pause();
    }
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn userspace_prog_1() {
    asm!("nop");
}
