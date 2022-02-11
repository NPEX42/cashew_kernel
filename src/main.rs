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
use cashew_kernel::{*, csh::{ShellArgs, ExitCode}};
use device::*;
use graphics_2d::*;
use x86_64::{VirtAddr, structures::paging::Size4KiB, PhysAddr};

#[cfg(not(test))]
entry_point!(kernel_main);

static mut FRAME: Frame = Frame::new();

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(mut fb) = boot_info.framebuffer.as_mut() {
        vga::initialize(fb.buffer_mut().as_mut_ptr(), fb.info());
        terminal::initialize();
        cashew_kernel::arch::initialize_interrupts();
        arch::enable_interrupts();

        pit::set_frequency(0, 60);

        input::init();
        let physical_memory_offset = boot_info.physical_memory_offset.into_option().unwrap();
        let phys_mem_offset = VirtAddr::new(physical_memory_offset);
        mem::setup_from(boot_info);
        mem::init(phys_mem_offset, &*boot_info.memory_regions);
        println!("Booting Complete, Press Any Key To continue");
        input::wait_for_key();

        



        

        csh::init();
        csh::exec("mount hdb");
        csh::main(Vec::new());
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


