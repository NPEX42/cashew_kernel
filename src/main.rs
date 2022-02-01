#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cashew_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{BootInfo};
#[cfg(not(test))]
use bootloader::entry_point;
use cashew_kernel::{*};
use graphics_2d::*;

#[cfg(not(test))]
entry_point!(kernel_main);

static mut FRAME: Frame = Frame::new();

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(mut fb) = boot_info.framebuffer.as_mut(){

        

        vga::initialize(fb.buffer_mut().as_mut_ptr(), fb.info());
        terminal::initialize();
        cashew_kernel::arch::initialize_interrupts();
        arch::enable_interrupts();

        pit::set_frequency(0, 60);

        input::ps2::PS2Controller::get().reinit().expect("[PS/2] - Initialization Failed...");

        #[cfg(test)]
        test_main();

        klog!("Goodbye{}", "!\n");
    }
    loop {cashew_kernel::arch::pause();}
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}


