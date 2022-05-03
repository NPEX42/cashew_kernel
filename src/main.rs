#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![test_runner(cashew_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use core::arch::asm;




use alloc::vec::Vec;
#[cfg(not(test))]
use bootloader::entry_point;
use bootloader::BootInfo;
use cashew_kernel::device::Pipe;
use cashew_kernel::vfs::drivers::simple_fat::fat::FileAttributeTable;
use cashew_kernel::{ata, graphics_2d::*, kerr, println, csh};

#[cfg(not(test))]
entry_point!(kernel_main);

static mut FRAME: Frame = Frame::new();

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(mut fb) = boot_info.framebuffer.as_mut() {
        cashew_kernel::boot(boot_info);


        println!("Booting Complete, Press Any Key To continue");


        let mut pipe = Pipe::new();
        pipe.write(0x80);
        println!("Pipe: {:02x?}", pipe.read());
        ata::cache_stats();

        let mut fat = FileAttributeTable::load(0, 4);

        for i in 0..fat.entry_count() {
            let entry = &fat[i];
            if !entry.is_empty() {
                println!("FAT[{}] = {}", i, entry);
            }
        }


        

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
