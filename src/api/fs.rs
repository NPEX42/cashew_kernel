use alloc::vec::Vec;

pub use crate::vfs::block::*;
pub use crate::vfs::drivers::{csh_fat, ustar};

pub fn read_to_vec(_name: &str) -> Option<Vec<u8>> {
    todo!();
}
