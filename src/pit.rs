use core::fmt::Display;

use crate::arch;

/// The PIT Is Clocked At 1.9318166 Mhz.
pub const PIT_BASE_FREQ: usize = 11931816666;

/// Ticks In One Second
static mut POLLING_FREQ: usize = 18;

#[derive(Debug, Copy, Clone, Default)]
pub struct Timer(u64);

impl Display for Timer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

static mut GLOBAL_TIMER: u64 = 0;

impl Timer {
    pub fn inc(&mut self) {
        self.0 += 1;
    }
}

pub fn update_timers() {
    unsafe {
        GLOBAL_TIMER += 1;
    }
}

pub fn uptime() -> u64 {
    unsafe { GLOBAL_TIMER }
}

pub fn set_frequency(channel: u8, frequency: u16) {
    arch::disable_interrupts();
    let command = ((channel & 0b11) << 6)
        | (0b11 << 4)  // LoByte/HighByte Transfer
        | (0b011 << 1) // Mode 2 - Freq. Divider
        | (0b0 << 0); // Binary Mode

    crate::sprint!("[PIT]: Command: 0b{:08b}\n", command);
    let reload = PIT_BASE_FREQ / frequency as usize;

    unsafe { POLLING_FREQ = frequency as usize }

    crate::arch::outb(0x43, command);
    crate::arch::outb(0x40 + (channel as u16), ((reload & 0x00FF) >> 0) as u8);
    crate::arch::outb(0x40 + (channel as u16), ((reload & 0xFF00) >> 8) as u8);
    arch::enable_interrupts();
}

pub fn sync() {
    sleep(1);
}

pub fn polling_rate() -> u64 {
    unsafe { POLLING_FREQ as u64 }
}

pub fn sleep_seconds(seconds: f32) {
    sleep((polling_rate() as f32 * seconds) as usize);
}

pub fn sleep(millis: usize) {
    let start = uptime();
    loop {
        let now = uptime();
        if now - start >= (millis as u64) {
            return;
        }
        crate::sprint!("");
        arch::x64::instructions::interrupts::enable_and_hlt();
    }
}
