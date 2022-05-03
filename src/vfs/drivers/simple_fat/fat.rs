use core::{ops::{Index, IndexMut, Range}, fmt::Display};

use alloc::{string::{String, ToString}, vec::{Vec}};

use crate::{ata::BLOCK_SIZE, api::fs::Block, klog};

use super::PhysicalBlockAddr;
/// The Size Of A Single Entry
pub const ENTRY_SIZE: usize = 32;
pub const ENTRIES_PER_BLOCK: usize = BLOCK_SIZE / ENTRY_SIZE;
pub const ENTRY_BIT_OFFSET: usize = usize::log2(ENTRY_SIZE) as usize;


/// 00..16: FileName,
/// 
/// 16..20: Begining Block,
/// 
/// 20..24: File Size,
/// 
/// 24..32: Reserved
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileEntry {
    name: String, 
    begin: PhysicalBlockAddr,
    size: u32,
}

pub struct FileAttributeTable {
    base: PhysicalBlockAddr,
    size: usize,
    entries: Vec<FileEntry>
}

impl FileEntry {
    pub fn empty() -> Self {
        Self {
            begin: 0,
            name: String::new(),
            size: 0,
        }
    }

    pub fn from_slice(data: &[u8]) -> Option<Self> {
        if data.len() < ENTRY_SIZE {
            klog!("Failed To Unpacked Entry. Data Len: {}", data.len());
            return None;
        }
        let name = String::from_utf8_lossy(&data[0..16]).to_string();
        let name = name.split_terminator("\0").nth(0).unwrap().into();
        let res = Self {
            name,
            begin: u32::from_be_bytes(data[16..20].try_into().expect("Conversion Failed")),
            size: u32::from_be_bytes(data[20..24].try_into().expect("Conversion Failed"))
        };
        Some(res)
    }

    pub fn to_slice(&self) -> [u8; ENTRY_SIZE] {
        let mut buffer = [0; ENTRY_SIZE];

        let name_bytes = &self.name.as_bytes()[0..16.min(self.name.len())];
        let size_bytes = self.size.to_be_bytes();
        let begin_bytes = self.begin.to_be_bytes();

        buffer[0..name_bytes.len()].copy_from_slice(name_bytes);
        buffer[16..20].copy_from_slice(&begin_bytes);
        buffer[20..24].copy_from_slice(&size_bytes);

        buffer
    }

    pub fn create_raw(name: &str, size: u32, begin: PhysicalBlockAddr) -> Self {
        Self {
            begin,
            name: name.to_string(),
            size
        }
    }

    pub fn block_addr_range(&self) -> Range<PhysicalBlockAddr> {
        let mut block_length = self.size / BLOCK_SIZE as u32;
        block_length += if self.size % BLOCK_SIZE as u32 > 0 {1} else {0}; 
        let end = self.begin + block_length;
        return self.begin..end;
    }

    pub fn is_empty(&self) -> bool {
        self.begin == 0
    }

    pub fn blocks(&self) -> Vec<Block> {
        let mut blocks = Vec::new();
        for addr in self.block_addr_range() {
            blocks.push(Block::read(addr).unwrap());
        }
        blocks
    }
}

impl Display for FileEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileEntry[{}, {}, {}]", self.name, self.size, self.begin)
    }
}

impl FileAttributeTable {
    pub fn load(base_address: PhysicalBlockAddr, amount: usize) -> Self {
        let mut entries = Vec::new();
        for block_addr in base_address..(amount as u32) + base_address {
            let block = Block::read(block_addr).expect("Failed To Read Block");
            for entry in 0..ENTRIES_PER_BLOCK {
                let entry_start = entry << 5;
                let entry_end = entry_start + ENTRY_SIZE;
                let slice = &block.data()[entry_start..entry_end];
                if let Some(entry_data) = FileEntry::from_slice(slice) {
                    entries.push(entry_data);
                }
            }
        }

        FileAttributeTable { entries, base: base_address, size: amount }
    }

    pub fn write(&self) {
        for block_addr in self.base..(self.size as u32 + self.base) {
            let mut block = Block::read(block_addr).unwrap();
            for entry in 0..ENTRIES_PER_BLOCK {
                let entry_start = entry << 5;
                let entry_end = entry_start + ENTRY_SIZE;

                block.data_mut()[entry_start..entry_end]
                    .copy_from_slice(
                        &self.entries[entry as usize + block_addr as usize * ENTRIES_PER_BLOCK]
                            .to_slice()
                );
            }

            block.write();

        }
    }

    pub fn entry_count(&self) -> usize {
        return self.entries.len();
    }
}

impl Index<usize> for FileAttributeTable {
    type Output = FileEntry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for FileAttributeTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}