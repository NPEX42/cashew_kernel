#![no_std]

#[no_mangle]
pub extern "C" fn main() {
    let mut i = 0;
    for _ in 0..1000 {
        i += 1;
    }
}