
pub unsafe fn read<T: Sized>(address: usize) -> T {
    let ptr = address as *const T;
    core::ptr::read_volatile::<T>(ptr)
}

pub unsafe fn read_u8(address: usize) -> u8 {
    read(address)
}

pub unsafe fn read_u16(address: usize) -> u16 {
    read(address)
}

pub unsafe fn read_u32(address: usize) -> u32 {
    read(address)
}

pub unsafe fn read_u64(address: usize) -> u64 {
    read(address)
}

pub unsafe fn write<T: Sized>(address: usize, item: T) {
    let ptr = address as *mut T;
    core::ptr::write_volatile(ptr, item);
}

pub unsafe fn write_u8(address: usize, val: u8) {
    write(address, val)
}

pub unsafe fn write_u16(address: usize, val: u8) {
    write(address, val)
}

pub unsafe fn write_u32(address: usize, val: u8) {
    write(address, val)
}

pub unsafe fn write_u64(address: usize, val: u8) {
    write(address, val)
}