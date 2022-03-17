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
use cashew_kernel::{*, csh::{ShellArgs, ExitCode}, arch::{cmos, vmm::{PTF_PRESENT_BIT, PTF_WRITABLE_BIT}}, mem::PTFlags, vfs::{drivers::{disk_map::DiskMap, csh_fat::{self, FAT, FileEntry, File}}, block::Block}, time::time};
use device::*;
use graphics_2d::*;
use x86_64::{VirtAddr, structures::paging::Size4KiB, PhysAddr};

#[cfg(not(test))]
entry_point!(kernel_main);

static mut FRAME: Frame = Frame::new();

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(mut fb) = boot_info.framebuffer.as_mut() {
        cashew_kernel::boot(boot_info);
        println!("Booting Complete, Press Any Key To continue");

        let block_count = 1;
        let start = time::ticks();
        for i in 0..block_count {
            Block::read((i as u32) % 16u32);
        }
        let end = time::ticks();
        let ticks = end - start;
        let ticks_per_block = ticks as f32 / block_count as f32;

        klog!("1x: {} ({} / Block)\n ", ticks, ticks_per_block );

        let block_count = 10;
        let start = time::ticks();
        for i in 0..block_count {
            Block::read((i as u32) % 16u32);
        }
        let end = time::ticks();
        let ticks = end - start;
        let ticks_per_block = ticks as f32 / block_count as f32;

        klog!("10x: {} ({} / Block)\n ", ticks, ticks_per_block );

        let block_count = 100;
        let start = time::ticks();
        for i in 0..block_count {
            Block::read((i as u32) % 16u32);
        }
        let end = time::ticks();
        let ticks = end - start;
        let ticks_per_block = ticks as f32 / block_count as f32;

        klog!("100x: {} ({} / Block)\n ", ticks, ticks_per_block );

        klog!("Write Tests ------------------");

        let block_count = 1;
        let start = time::ticks();
        for i in 0..block_count {
            Block::empty((i as u32) % 16u32).write();
        }
        let end = time::ticks();
        let ticks = end - start;
        let ticks_per_block = ticks as f32 / block_count as f32;

        klog!("1x: {} ({} / Block)\n ", ticks, ticks_per_block );

        let block_count = 10;
        let start = time::ticks();
        for i in 0..block_count {
            Block::empty((i as u32) % 16u32).write();
        }
        let end = time::ticks();
        let ticks = end - start;
        let ticks_per_block = ticks as f32 / block_count as f32;

        klog!("10x: {} ({} / Block)\n ", ticks, ticks_per_block );

        let block_count = 100;
        let start = time::ticks();
        for i in 0..block_count {
            Block::empty((i as u32) % 16u32).write();
        }
        let end = time::ticks();
        let ticks = end - start;
        let ticks_per_block = ticks as f32 / block_count as f32;

        klog!("100x: {} ({} / Block)\n ", ticks, ticks_per_block );



        csh::main(Vec::new());
        cashew_kernel::shutdown();
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


pub unsafe extern "C" fn userspace_prog_1() {
    asm!("nop");
}


