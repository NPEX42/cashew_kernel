pub use crate::device::{info, is_mounted, mount, read_block, write_block, BlockDeviceIO};

pub mod funcs {
    pub type ChrReadFn      = extern "C" fn() -> u8;
    pub type ChrWriteFn     = extern "C" fn(u8);
    pub type ChrIdentifyFn  = extern "C" fn() -> (*const u8, u8); 
}
use alloc::string::String;
use funcs::*;
pub struct CharDeviceDriver {
    read_fn:        funcs::ChrReadFn,
    write_fn:       funcs::ChrWriteFn,
    identify_fn:   funcs::ChrIdentifyFn
}

impl CharDeviceDriver {
    pub fn new(read_fn: ChrReadFn, write_fn: ChrWriteFn, identify_fn: ChrIdentifyFn) -> Self {
        Self {
            identify_fn,
            read_fn,
            write_fn
        }
    }

    pub unsafe fn read_u8(&self) -> u8 {
        (self.read_fn)()
    }

    pub unsafe fn write_u8(&self, val: u8) {
        (self.write_fn)(val)
    }

    pub unsafe fn identify(&self) -> String {
        let (bytes, len) = (self.identify_fn)();
        let bytes = core::slice::from_raw_parts(bytes, len as usize);
        String::from_utf8_lossy(bytes).into()
    }
}

