use core::fmt::Display;

use alloc::{string::String, vec::Vec, boxed::Box};

use crate::{
    device::{self, BlockAddr},
    vfs::block::Block,
};


use super::{FileIO, FileWrite, FileRead, VirtFileSystem, FileAppend};

#[derive(PartialEq, Debug, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
#[repr(u8)]
pub enum FileType {
    Normal = 0,
    HardLink = 1,
    SymLink = 2,
    CharDev = 3,
    BlockDev = 4,
    Directory = 5,
    FIFO = 6,
    Contigous = 7,

    #[default]
    Unknown = 255,
}

impl FileType {
    pub fn from_u8(b: u8) -> Self {
        match b {
            0 => Self::Normal,
            1 => Self::HardLink,
            2 => Self::SymLink,
            3 => Self::CharDev,
            4 => Self::BlockDev,
            5 => Self::Directory,
            6 => Self::FIFO,
            7 => Self::Contigous,

            _ => Self::Unknown,
        }
    }
}

pub struct FileSystem;

impl VirtFileSystem for FileSystem {
    fn open_file(&self, filename: &str) -> Option<alloc::boxed::Box<dyn FileIO>> {
        if let Ok(file_info) = FileInfo::open(filename) {
            Some(Box::new(file_info))
        } else {
            return None;
        }
    }
}

#[derive(Debug)]
pub struct FileInfo {
    name: String,
    blocks: Vec<Block>,
    size: u32,
    filetype: FileType,
}

impl FileInfo {
    pub fn to_string(&self) -> String {
        String::from_utf8(self.to_vec()).expect("Failed To Load File To String...")
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut v = Vec::new();
        let mut count = 0;
        'block_loop: for block in self.blocks() {
            for byte in block.data() {
                v.push(*byte);
                count += 1;

                if count > self.size {
                    break 'block_loop;
                }
            }
        }
        v
    }

    pub fn open(name: &str) -> Result<FileInfo, ()> {
        let max = device::info()?.blocks;
        let mut address = 0;

        while address < max {
            let entry = Self::load(address as u32)?;

            if entry.name().eq(name) && entry.filetype() == FileType::Normal {
                return Ok(entry);
            }

            address += entry.data_block_count() + 1;
        }

        Err(())
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn data_block_count(&self) -> usize {
        self.size() / 512 + if self.size() % 512 > 0 { 1 } else { 0 }
    }

    pub fn filetype(&self) -> FileType {
        self.filetype
    }

    pub fn load(addr: BlockAddr) -> Result<FileInfo, ()> {
        let info = Block::read(addr).unwrap();

        //sprint!("Loaded Block #{}\n",addr);

        let mut name_end = 0;

        for _ in 0..100 {
            if info.data()[name_end] == 0 {
                break;
            }

            name_end += 1;
        }

        let name = String::from(String::from_utf8_lossy(&info.data()[0..name_end].to_vec()).trim());
        let size = String::from_utf8(info.data()[124..135].to_vec()).unwrap_or_default();
        //sprint!("Size: '{}'\n", size);
        let size: u32 = u32::from_str_radix(&size, 8).unwrap_or(0);
        //sprint!("Size: {} Bytes\n", size);
        let mut blocks = Vec::new();
        let block_len = (size / 512) + if size % 512 > 0 { 1 } else { 0 };

        let ty: u8 =
            u8::from_str_radix(&String::from_utf8_lossy(&info.data()[156..157].to_vec()), 8)
                .unwrap_or(255);

        for i in 1..=block_len {
            blocks.push(Block::read(addr + i).unwrap());
        }
        Ok(Self {
            name,
            blocks,
            size,
            filetype: FileType::from_u8(ty),
        })
    }

    pub fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }

    pub fn is_file(&self) -> bool {
        self.filetype == FileType::Normal
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for FileInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:>4} - {}", self.size(), self.name())
    }
}

impl FileWrite for FileInfo {
    fn write(&mut self, _: usize, _: u8) {
        unimplemented!("USTAR IS READ ONLY")
    }
}

impl FileAppend for FileInfo {
    fn append(&mut self, _: u8) {
        unimplemented!("READ ONLY")
    }

    fn append_vec(&mut self, _: Vec<u8>) {
        unimplemented!("READ ONLY")
    }

    fn append_bytes(&mut self, _: &[u8]) {
        unimplemented!("READ ONLY")
    }
}

impl FileRead for FileInfo {
    fn read(&self, index: usize) -> u8 {
        self.to_vec()[index]
    }
    
    fn read_to_string(&self) -> String {
        self.to_string()
    }

    fn read_bytes(&self, buffer: &mut [u8]) -> usize {
        let temp_data = self.to_vec();
        let size = buffer.len().min(temp_data.len());
        buffer[..size].copy_from_slice(&temp_data[..size]);
        return size;
    }

    fn read_to_vec(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl FileIO for FileInfo {
    fn close(&mut self) {}
    fn rename(&mut self, _: &str) {
        unimplemented!("READ-ONLY")
    }

    fn size(&self) -> usize {
        self.size as usize
    }
    
}