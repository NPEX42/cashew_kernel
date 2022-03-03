use alloc::string::String;

use crate::vfs::block::{self, Block};

pub const FAT_START: usize = 1;
pub const FAT_SIZE: usize = 256;
pub const ENTRY_SIZE: usize = 32;
pub const FNAME_SIZE: usize = 16;
pub const TYPE_SIZE: usize = 1;

pub struct FAT;

impl FAT {
    pub fn get_entry(index: usize) -> Result<FileEntry, ()> {
        let block = Block::read((index / ENTRY_SIZE) as u32)?;

        let name = String::from_utf8_lossy(block[])
        
    }
}





#[derive(Debug, Clone, Default)]
pub struct FileEntry {
    pub name: String,
    pub size: u32,
    pub ftype: u8,
}