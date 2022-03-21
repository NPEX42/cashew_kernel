pub unsafe fn read<T: Sized>(index: usize) -> T {
    let ptr = index as *const T;
    core::ptr::read_volatile::<T>(ptr)
}

pub unsafe fn read_u8(index: usize) -> u8 {
    read(index)
}

pub unsafe fn read_u16(index: usize) -> u16 {
    read(index)
}

pub unsafe fn read_u32(index: usize) -> u32 {
    read(index)
}

pub unsafe fn read_u64(index: usize) -> u64 {
    read(index)
}

pub unsafe fn write<T: Sized>(index: usize, item: T) {
    let ptr = index as *mut T;
    core::ptr::write_volatile(ptr, item);
}

pub unsafe fn write_u8(index: usize, val: u8) {
    write(index, val)
}

pub unsafe fn write_u16(index: usize, val: u8) {
    write(index, val)
}

pub unsafe fn write_u32(index: usize, val: u8) {
    write(index, val)
}

pub unsafe fn write_u64(index: usize, val: u8) {
    write(index, val)
}
