use x86_64::{instructions::port::Port, structures::port::{PortWrite, PortRead}};

pub fn write<T: Copy + PortWrite>(port: u16, value: T) {
    let mut p: Port<T> = Port::new(port);
    unsafe { p.write(value) };
}

pub fn read<T: Copy + PortRead>(port: u16) -> T{
    let mut p: Port<T> = Port::new(port);
    unsafe { p.read() }
}
