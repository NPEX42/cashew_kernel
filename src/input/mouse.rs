use conquer_once::spin::OnceCell;
use ps2_mouse::*;

use crate::{locked::Locked, arch::{inb, enable_interrupts}, klog, sprint};
static MOUSE: OnceCell<Locked<Mouse>> = OnceCell::uninit();

static mut MOUSE_X: usize = 0;
static mut MOUSE_Y: usize = 0;

pub fn init() {
    klog!("Initializing Mouse - Step 1/3\n");
    MOUSE.init_once(|| {Locked::new(Mouse::new())});

    reset();
    //set_handler();
}

fn reset() {

    sprint!("==== RESET ====\n");

    if let Some(mouse) = MOUSE.get() {
        mouse.force_unlock();
        let mouse = &mut mouse.lock();
        sprint!("==== LOCKED ====\n");
        enable_interrupts();
        mouse.init().expect("FAILED");
        sprint!("==== INITED ====\n");
        klog!("Initialized Mouse\n");
    } else {
        klog!("Failed To Initialize - Mouse Uninited...\n");
    }
}

fn set_handler() {
    sprint!("==== HANDLER ====\n");
    if let Some(mouse) = MOUSE.get() {
        mouse.force_unlock();
        let mouse = &mut mouse.lock();
        mouse.set_on_complete(on_complete);
        klog!("Handler Set\n");
    } else {
        klog!("Failed To Set Handler - Mouse Uninited...\n");
    }
}

pub(crate) fn update() {
    let byte = inb(0x60);
    MOUSE.get().unwrap().lock().process_packet(byte);
}

fn on_complete(state: MouseState) {
    let mut mx = unsafe { MOUSE_X as i16 };
    let mut my = unsafe { MOUSE_Y as i16 };
    mx += state.get_x();
    my += state.get_y();

    mx = mx.clamp(0, 640);
    my = my.clamp(0, 480);

    unsafe {MOUSE_X = mx as usize;}
    unsafe {MOUSE_Y = my as usize;}
}