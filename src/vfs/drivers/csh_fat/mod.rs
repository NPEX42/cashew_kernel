mod bitmap;
mod api;
mod inode;
mod index_block;
pub mod file;

pub use api::*;

use core::fmt::Display;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use bit_field::BitField;

use crate::{
    klog,
    vfs::{
        block::{Block, LinkedBlock},
        BlockAllocator,
    },
};
pub const FAT_START: usize = 1;
pub const FAT_SIZE: usize = 256;
pub const ENTRY_SIZE: usize = 32;
pub const DISK_SIZE: usize = 128 << 20;
pub const DISK_BLOCKS: usize = DISK_SIZE / 512;

pub mod superblock;

pub const FNAME_START: usize = 0;
pub const FNAME_SIZE: usize = 16;

pub const SIZE_START: usize = FNAME_START + FNAME_SIZE;
pub const SIZE_SIZE: usize = 4;

pub const TYPE_START: usize = SIZE_START + SIZE_SIZE;
pub const TYPE_SIZE: usize = 1;

pub const DSTART_START: usize = TYPE_START + TYPE_SIZE;
pub const DSTART_SIZE: usize = 4;

pub const DATA_START: usize = FAT_START + FAT_SIZE + (DISK_SIZE / 4096);

pub struct BlockBitmap;

impl BlockBitmap {
    fn get_bitmap_offset(block: usize) -> usize {
        block - DATA_START
    }

    fn get_bitmap_index(block: usize) -> u32 {
        (((block - DATA_START) / 4096) + FAT_START + FAT_SIZE) as u32
    }

    fn alloc(block: usize) {
        let mut blck = Block::read(Self::get_bitmap_index(block)).unwrap();
        let offset = Self::get_bitmap_offset(block);
        blck.data_mut()[offset / 8].set_bit(offset & 7, true);
        blck.write();
    }

    fn dealloc(block: usize) {
        let mut blck = Block::read(Self::get_bitmap_index(block)).unwrap();
        let offset = Self::get_bitmap_offset(block);
        blck.data_mut()[offset / 8].set_bit(offset & 7, false);
        blck.write();
    }

    fn is_used(block: usize) -> bool {
        let mut blck = Block::read(Self::get_bitmap_index(block)).unwrap();
        let offset = Self::get_bitmap_offset(block);
        blck.data_mut()[offset / 8].get_bit(offset & 7)
    }

    pub fn get() -> Self {
        BlockBitmap
    }
}

impl BlockAllocator for BlockBitmap {
    fn allocate(&mut self) -> Option<Block> {
        for block in DATA_START..DISK_BLOCKS {
            if !Self::is_used(block) {
                if let Some(blck) = Block::read(block as u32) {
                    Self::alloc(block);
                    return Some(blck);
                } else {
                    return None;
                }
            }
        }
        return None;
    }

    fn free(&mut self, block: u32) {
        Self::dealloc(block as usize)
    }
}

pub struct FAT;

impl FAT {
    pub fn entry_count() -> usize {
        FileEntryIter::default().count()
    }

    pub fn next_free() -> Option<usize> {
        for index in 0..FAT_SIZE / (512 / ENTRY_SIZE) {
            match FAT::get_entry(index) {
                Ok(_) => {}
                Err(_) => return Some(index),
            }
        }

        return None;
    }

    pub fn get_entry(index: usize) -> Result<FileEntry, ()> {
        let block = Block::read(((index / ENTRY_SIZE) + FAT_START) as u32).unwrap();
        let start = (index & 0xF) << 5;
        let end = start + ENTRY_SIZE;

        if start == 512 {
            return Err(());
        }

        let entry_data = &block.data()[start..end];

        let mut entry: FileEntry = Default::default();

        if entry_data[0] == 0 {
            return Err(());
        }

        entry.name = String::from_utf8_lossy(&entry_data[FNAME_START..SIZE_START])
            .trim()
            .to_string();
        entry.size = u32::from_be_bytes(entry_data[SIZE_START..TYPE_START].try_into().unwrap());
        entry.ftype = entry_data[TYPE_START];
        entry.data_start = u32::from_be_bytes(
            entry_data[DSTART_START..DSTART_START + 4]
                .try_into()
                .unwrap(),
        );
        Ok(entry)
    }

    pub fn set_entry(index: usize, entry: Option<&FileEntry>) -> Result<(), ()> {
        let mut block = Block::read(((index / ENTRY_SIZE) + FAT_START) as u32).unwrap();
        let start = index << 5;
        let end = start + ENTRY_SIZE;
        let entry_data = &mut block.data_mut()[start..end];

        if let Some(entry) = entry {
            assert!(entry.name.as_bytes().len() <= 16);
            entry_data[0..FNAME_SIZE].fill(b' ');
            entry_data[0..entry.name.as_bytes().len()].copy_from_slice(entry.name.as_bytes());
            entry_data[SIZE_START..TYPE_START].copy_from_slice(&entry.size.to_be_bytes());
            entry_data[TYPE_START] = entry.ftype;
            entry_data[DSTART_START..DSTART_START + DSTART_SIZE]
                .copy_from_slice(&entry.data_start.to_be_bytes());
        } else {
            entry_data.fill(0);
        }

        block.write();

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct FileEntryIter {
    next: usize,
}

impl Iterator for FileEntryIter {
    type Item = (usize, FileEntry);

    fn next(&mut self) -> Option<Self::Item> {
        while let Err(_) = FAT::get_entry(self.next) {
            self.next += 1;
            if self.next >= FAT_SIZE / (512 / ENTRY_SIZE) {
                return None;
            };
        }

        let r = FAT::get_entry(self.next).expect("Failed To Get Entry...");

        self.next += 1;

        Some((self.next - 1, r))
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileEntry {
    pub name: String,
    pub size: u32,
    pub ftype: u8,

    pub data_start: u32,
}
#[allow(deprecated)]
impl FileEntry {
    pub fn to_vec(&self) -> Result<Vec<u8>, ()> {
        let mut block = LinkedBlock::read(self.data_start)?;
        Ok(block.to_vec_sized(self.size as usize))
    }

    pub fn set_data(&mut self, data: &[u8]) -> Result<(), ()> {
        self.size = data.len() as u32;

        if let Some(mut head) = self.head() {
            klog!("Found Head ({})...\n", head.addr());
            head.set_data(data).expect("Failed To Set Data");
        } else {
            klog!("No Head Found...\n");
            self.data_start = BlockBitmap::get().allocate().unwrap().addr();
            let mut head = self.head().unwrap();
            head.set_data(data).expect("Failed To Set Data");
        }

        Ok(())
    }

    pub fn head(&self) -> Option<LinkedBlock> {
        if self.data_start == 0 {
            return None;
        } else {
            Some(LinkedBlock::read(self.data_start).expect("Failed To Read Block"))
        }
    }
}

impl Display for FileEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(head) = self.head() {
            write!(
                f,
                "FAT Entry [{:<16}|{} Bytes|{}]",
                self.name, self.size, head
            )
        } else {
            write!(f, "FAT Entry [{:<16}|{}|NUL]", self.name, self.name)
        }
    }
}

pub struct File {
    entry: (usize, FileEntry),
    data: Vec<u8>,
    pos: usize,
}

impl File {
    pub fn new(name: &str) -> Option<Self> {
        if let Some(index) = FAT::next_free() {
            let mut entry = FileEntry::default();
            entry.name = name.into();

            FAT::set_entry(index, Some(&entry)).expect("Failed To Create New Entry...");

            Some(Self {
                data: Vec::new(),
                pos: 0,
                entry: (index, entry),
            })
        } else {
            None
        }
    }

    fn from(index: usize, file_entry: FileEntry) -> Self {
        Self {
            data: file_entry.to_vec().expect("Failed To Read Data"),
            entry: (index, file_entry),
            pos: 0,
        }
    }

    pub fn open(name: &str) -> Option<Self> {
        let files = FileEntryIter::default();
        for (index, file) in files {
            if file.name == name {
                return Some(File::from(index, file));
            }
        }

        return None;
    }

    fn sync(&mut self) -> Result<(), ()> {
        let entry = &mut self.entry.1;
        let index = self.entry.0;

        entry.set_data(&self.data)?;

        FAT::set_entry(index, Some(entry))?;
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), ()> {
        self.sync()
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        for i in 0..data.len() {
            if self.pos >= self.data.len() {
                self.sync().expect("Failed To Sync File Data");
                return i;
            };
            self.data[self.pos] = data[i];
            self.pos += 1;
        }
        self.sync().expect("Failed To Sync File Data");
        data.len()
    }
}

impl Display for File {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} [{}]", self.entry.1.name, self.entry.1)
    }
}
