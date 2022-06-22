use core::{ops::{Index, IndexMut, Range, Sub, Add}, fmt::{Display, Debug}};

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

    index: usize,
    data: Vec<u8>,
    is_empty: bool,
}


pub struct FileAttributeTable {
    base: PhysicalBlockAddr,
    size: usize,
    entries: Vec<FileEntry>
}

impl FileEntry {

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn empty(index: usize) -> Self {
        Self {
            begin: 0,
            name: String::new(),
            size: 0,
            index,
            data: Vec::new(),
            is_empty: true,
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
            data: Vec::new(),
            is_empty: (data[0] == 0)
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

    pub fn create_raw(index:usize, name: &str, size: u32, begin: PhysicalBlockAddr, is_empty: bool) -> Self {
        Self {
            begin,
            name: name.to_string(),
            size,
            index,
            data: Vec::new(),
            is_empty
        }
    }

    pub fn block_addr_range(&self) -> Range<PhysicalBlockAddr> {
        let mut block_length = self.size / BLOCK_SIZE as u32;
        block_length += if self.size % BLOCK_SIZE as u32 > 0 {1} else {0}; 
        let end = self.begin + block_length;
        return self.begin..end;
    }

    /// An Entry Is Considered Empty If `Begin` is 0.
    pub fn is_empty(&self) -> bool {
        self.is_empty
    }

    pub fn blocks(&self) -> Vec<Block> {
        let mut blocks = Vec::new();
        for addr in self.block_addr_range() {
            blocks.push(Block::read(addr).unwrap());
        }
        blocks
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.into();
    }

    pub fn erase(&mut self) {
        self.begin = 0;
        self.size = 0;
        self.set_name("\0");
        self.mark_free();
        self.set_data(&[]);

    }

    pub fn mark_used(&mut self) {
        self.is_empty = false;
    }

    pub fn mark_free(&mut self) {
        self.is_empty = true;
    }

    pub fn set_data(&mut self, data: &[u8]) {
        self.size = data.len().try_into().expect("Failed To Cast usize -> u32");

    

        let mut fat = super::FILE_SYSTEM.lock();
        if self.size > 0 {
            if let Some(new_begin) = fat.find_free_range(data.len()) {
                self.begin = new_begin;
            }

            for (index, chunk) in data.chunks(BLOCK_SIZE).enumerate() {
                let mut block = Block::read(index as u32 + self.begin).expect("No");
                block.data_mut()[0..chunk.len()].clone_from_slice(chunk);
                block.write();
            }
        }

        fat[self.index] = self.clone();

        fat.write();
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn to_vec(&self) -> Vec<u8> {

        let size = self.size as usize;
        let mut buffer = Vec::new();

        'outer: for block in self.blocks() {

            let bytes = block.data();

            for i in 0..bytes.len() {

                if buffer.len() >= size {
                    break 'outer;
                }

                buffer.push(bytes[i]);

            }

        }

        buffer
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
        let mut fat_index = 0;
        for block_addr in base_address..(amount as u32) + base_address {
            let block = Block::read(block_addr).expect("Failed To Read Block");
            for entry in 0..ENTRIES_PER_BLOCK {
                let entry_start = entry << 5;
                let entry_end = entry_start + ENTRY_SIZE;
                let slice = &block.data()[entry_start..entry_end];
                if let Some(entry_data) = FileEntry::from_slice(fat_index * ENTRIES_PER_BLOCK + entry, slice) {
                    //klog!("Loaded Entry: {:?}", entry_data);
                    entries.push(entry_data);
                }
            }
            fat_index += 1;
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

    pub fn next_free_index(&self) -> Option<usize> {
        for (idx, entry) in self.entries.iter().enumerate() {
            if entry.is_empty() {
                return Some(idx);
            }
        }

        None
    }

    pub fn next_free_entry(&self) -> Option<FileEntry> {
        match self.next_free_index() {
            None => None,
            Some(index) => Some(self[index].clone()),
        }
    }


    pub fn find_free_range(&self, size: usize) -> Option<PhysicalBlockAddr> {
        let blocks = Self::bytes_to_blocks(size);

        for addr in 4..device::blk_dev_size() {
            let target_range = addr as u32..(addr + blocks) as u32;
            let mut is_range_free = true;
            for entry in &self.entries {
                let range = entry.block_addr_range();
                if range.is_empty() {continue;}
                klog!("Entry Range: {:?}", range);
                klog!("Target Range: {:?}", target_range);
                if range_overlaps(&range, &target_range) {
                    klog!("Range Is Used.");
                    is_range_free = false;
                } else {
                    //klog!("Range Is Free.");
                }
            }

            if is_range_free {
                return Some(target_range.start);
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



/// Returns True if inner overlaps outer;
/// 
/// That is if `r0.start >= r1.start` OR  `r0.end <= r1.end`.
fn range_overlaps<T: Ord + Sub<T, Output = T> + Add<T, Output = T> + Copy + Debug>(inner: &Range<T>, outer: &Range<T>) -> bool {
    
    let min = inner.start.min(outer.start);
    let max = inner.end.max(outer.end);
    let sum_of_ranges = (inner.end - inner.start) + (outer.end - outer.start);
    //klog!("Sum Of Ranges: {:?}", sum_of_ranges);
    let max_min_diff = max - min;
    //klog!("Max - Min = {:?}", max_min_diff);
    sum_of_ranges > max_min_diff

}