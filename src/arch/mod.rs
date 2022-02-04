pub use x86_64 as x64;

use x64::instructions::port::Port;

mod idt;
mod pic;

pub fn initialize_interrupts() {
    idt::initialize();
    pic::initialize();
}

pub fn spin() {
    pause()
}

pub fn outb(port: u16, data: u8) {
    unsafe {
        let mut port: Port<u8> = Port::new(port);
        port.write(data)
    }
}

pub fn outw(port: u16, data: u16) {
    unsafe {
        let mut port: Port<u16> = Port::new(port);
        port.write(data)
    }
}

pub fn outl(port: u16, data: u32) {
    unsafe {
        let mut port: Port<u32> = Port::new(port);
        port.write(data)
    }
}

pub fn inb(port: u16) -> u8 {
    unsafe {
        let mut port: Port<u8> = Port::new(port);
        port.read()
    }
}

pub fn inw(port: u16) -> u16 {
    unsafe {
        let mut port: Port<u16> = Port::new(port);
        port.read()
    }
}

pub fn inl(port: u16) -> u32 {
    unsafe {
        let mut port: Port<u32> = Port::new(port);
        port.read()
    }
}

pub fn disable_interrupts() {
    crate::sprint!("[Arch]: Disabling Interrupts\n");
    x64::instructions::interrupts::disable();
}

pub fn enable_interrupts() {
    crate::sprint!("[Arch]: Enabling Interrupts\n");
    x64::instructions::interrupts::enable();
}

pub fn pause() {
    x64::instructions::interrupts::enable_and_hlt();
}

#[cfg(feature = "breakpoints")]
#[macro_export]
macro_rules! breakpoint {
    () => {
        $crate::sprint!("Breakpoint @ {}:{}:{}", file!(), line!(), column!());
        $crate::arch::x64::instructions::interrupts::int3();
    };
}

#[cfg(not(feature = "breakpoints"))]
#[macro_export]
macro_rules! breakpoint {
    () => {};
}
