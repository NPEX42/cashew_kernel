#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};
use cashew_kernel::{*, pit::uptime, fonts::FONT_HEIGHT};
use graphics_2d::*;
use pc_keyboard::KeyCode;
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

        pit::set_frequency(0, 60);


        let LINES = 80 * 60;

        let start_time = uptime();
        let mut last= start_time;
        terminal::set_print_newline(true);
        terminal::swap();
        loop {
            if let Some(key) = input::keyboard::read_char() {
                print!("{}", key);
                input::keyboard::clear();
            } else if let Some(kc) = input::keyboard::read_keycode() {
                match kc {

                    KeyCode::F1 => {terminal::set_bg(Pixel::hex(0xFF0000))},
                    KeyCode::F2 => {terminal::set_bg(Pixel::hex(0x00FF00))},
                    KeyCode::F3 => {terminal::set_bg(Pixel::hex(0x0000FF))},

                    KeyCode::F4 => {
                        terminal::clear();
                        terminal::home();
                    },

                    KeyCode::ArrowDown => {
                        terminal::move_y(FONT_HEIGHT as isize);
                    },
                    _ => {}
                }

                input::keyboard::clear();
            }
        }


        let end_time = uptime();

        let time_taken_seconds = (end_time - start_time) as f32 / pit::polling_rate() as f32;
        println!();
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