use conquer_once::spin::OnceCell;
use pic8259::ChainedPics;

use crate::locked::Locked;

static PICS: OnceCell<Locked<ChainedPics>> = OnceCell::uninit();
pub type IrqIndex = u8;

pub const PIC1: IrqIndex = 0x20;
pub const PIC2: IrqIndex = PIC1 + 8;

pub fn initialize() {
    PICS.init_once(|| Locked::new(unsafe { ChainedPics::new(PIC1, PIC2) }));

    unsafe {
        PICS.get().unwrap().lock().initialize();
    }
}

pub fn notify_eoi(irq: IrqIndex) {
    if let Some(pics) = PICS.get() {
        unsafe {
            pics.lock().notify_end_of_interrupt(irq);
        }
    }
}
