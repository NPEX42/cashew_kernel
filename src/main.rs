#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cashew_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use alloc::vec::Vec;
#[cfg(not(test))]
use bootloader::entry_point;
use bootloader::BootInfo;
use cashew_kernel::{*};
use device::*;
use graphics_2d::*;
use x86_64::VirtAddr;

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
        mem::init(phys_mem_offset, &*boot_info.memory_regions);

        device::mount(Device::hda());

        csh::init();
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
