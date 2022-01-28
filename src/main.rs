#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};
use cashew_kernel::{*, pit::uptime};
use graphics_2d::*;
use core::panic::PanicInfo;

use cashew_kernel::sprint;

entry_point!(kernel_main);

static mut FRAME: Frame = Frame::new();

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(mut fb) = boot_info.framebuffer.as_mut(){
        vga::initialize(fb.buffer_mut().as_mut_ptr(), fb.info());
        terminal::initialize();
        cashew_kernel::arch::initialize_interrupts();
        arch::enable_interrupts();

        pit::set_frequency(0, 1000);


        let LINES = 10000;

        let start_time = uptime();
        for i in 0..=LINES {
            println!("Hello! - #{:05}", i);
        }

        let end_time = uptime();

        let time_taken_seconds = (end_time - start_time) as f32 / pit::polling_rate() as f32;
        println!("Time Taken: {:02.3} Seconds - {:02.3} Lines / Second.", time_taken_seconds, LINES as f32 / time_taken_seconds);
        


        terminal::set_x(0);
        terminal::set_y(0);

        

    }
    loop {cashew_kernel::arch::pause();}
}




#[panic_handler]
fn panic(info: &PanicInfo) -> ! {

    sprint!("Panic: {}\n", info);
    //kerr!("== Kernel Panic ==\n{}", info);
    loop {}
}