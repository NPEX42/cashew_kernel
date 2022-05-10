pub mod block;
pub mod drivers;
use alloc::boxed::Box;

use crate::vfs::drivers::FileIO;

use self::drivers::{
    VirtFileSystem, 
    ustar
};

pub fn open_file(filename: &str) -> Option<Box<dyn FileIO>> {
    ustar::FileSystem::open_file(filename)
}