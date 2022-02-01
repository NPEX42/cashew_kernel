pub mod keyboard;
pub mod mouse;

pub mod ps2;

pub fn init() {
    keyboard::initialize();
    ps2::PS2Controller::get().reinit().expect("[PS/2] - Initialization Failed...");
    mouse::init();
}