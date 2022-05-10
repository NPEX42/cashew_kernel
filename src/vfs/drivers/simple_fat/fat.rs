use core::{ops::{Index, IndexMut, Range}, fmt::Display};

use alloc::{string::{String, ToString}, vec::{Vec}};

use crate::{ata::BLOCK_SIZE, api::fs::Block, klog, device};

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

    index: usize
}

pub struct FileAttributeTable {
    base: PhysicalBlockAddr,
    size: usize,
    entries: Vec<FileEntry>
}

impl FileEntry {
    pub fn empty(index: usize) -> Self {
        Self {
            begin: 0,
            name: String::new(),
            size: 0,
            index,
        }
    }

    pub fn from_slice(index: usize, data: &[u8]) -> Option<Self> {
        if data.len() < ENTRY_SIZE {
            klog!("Failed To Unpacked Entry. Data Len: {}", data.len());
            return None;
        }
        let name = String::from_utf8_lossy(&data[0..16]).to_string();
        let name = name.split_terminator("\0").nth(0).unwrap().into();
        let res = Self {
            name,
            begin: u32::from_be_bytes(data[16..20].try_into().expect("Conversion Failed")),
            size: u32::from_be_bytes(data[20..24].try_into().expect("Conversion Failed")),
            index,
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

    pub fn create_raw(index:usize, name: &str, size: u32, begin: PhysicalBlockAddr) -> Self {
        Self {
            begin,
            name: name.to_string(),
            size,
            index,
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

    pub fn set_data(&mut self, data: &[u8]) {
        self.size = data.len().try_into().expect("Failed To Cast usize -> u32");
        let mut fat = FileAttributeTable::default();
        if let Some(new_begin) = fat.find_free_range(data.len()) {
            self.begin = new_begin;
        }

        for (index, chunk) in data.chunks(BLOCK_SIZE).enumerate() {
            let mut block = Block::read(index as u32 + self.begin).expect("No");
            block.data_mut()[0..chunk.len()].clone_from_slice(chunk);
            block.write();
        }

        fat[self.index] = self.clone();

        fat.write();
    }

    pub fn contains_block(&self, addr: PhysicalBlockAddr) -> bool {
        self.block_addr_range().contains(&addr)
    }
}

impl Display for FileEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileEntry[{}, {}, {}]", self.name, self.size, self.begin)
    }
}

impl FileAttributeTable {

    pub fn default() -> Self {
        Self::load(0, 4)
    }

    pub fn load(base_address: PhysicalBlockAddr, amount: usize) -> Self {
        let mut entries = Vec::new();
        for block_addr in base_address..(amount as u32) + base_address {
            let block = Block::read(block_addr).expect("Failed To Read Block");
            for entry in 0..ENTRIES_PER_BLOCK {
                let entry_start = entry << 5;
                let entry_end = entry_start + ENTRY_SIZE;
                let slice = &block.data()[entry_start..entry_end];
                if let Some(entry_data) = FileEntry::from_slice(entry, slice) {
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

    pub fn search_for_file(&self, name: &str) -> Option<&FileEntry> {
        for entry in &self.entries {
            if entry.name.eq(name) {
                return Some(entry);
            }
        }
        None
    }

    pub fn search_for_file_mut(&mut self, name: &str) -> Option<&mut FileEntry> {
        for entry in &mut self.entries {
            if entry.name.eq(name) {
                return Some(entry);
            }
        }
        None
    }


    pub fn find_free_range(&self, size: usize) -> Option<PhysicalBlockAddr> {
        let blocks = Self::bytes_to_blocks(size);

        for addr in 4..device::blk_dev_size() {
            let target_range = addr as u32..(addr + blocks) as u32;
            for entry in &self.entries {
                let range = entry.block_addr_range();
                klog!("Entry Range: {:?}", range);
                klog!("Target Range: {:?}", target_range);
                if range_overlaps(&range, &target_range) {
                    klog!("Range Is Used.");
                    break;
                } else {
                    klog!("Range Is Free.");
                   return Some(addr as u32);
                }
            }

        }

        return None;
    }

    fn bytes_to_blocks(size: usize) -> usize {
        let mut blocks = size / BLOCK_SIZE;
        if size % BLOCK_SIZE > 0 {
            blocks += 1;
        };

        return blocks;
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


/// Returns True if r0 is a subset of r1, or if r1 contains r0.
/// 
/// That is if `r0.start >= r1.start` AND  `r0.end <= r1.end`.
fn range_is_subset<T: PartialOrd>(r0: &Range<T>, r1: &Range<T>) -> bool {
    (r0.start >= r1.start) && (r0.end <= r1.end)
}

/// Returns True if inner overlaps outer;
/// 
/// That is if `r0.start >= r1.start` OR  `r0.end <= r1.end`.
fn range_overlaps<T: PartialOrd>(inner: &Range<T>, outer: &Range<T>) -> bool {
    range_is_subset(inner, outer) || (inner.contains(&outer.start)) ^ (inner.contains(&outer.end))
}