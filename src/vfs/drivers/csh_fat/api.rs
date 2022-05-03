use core::ops::{Index, IndexMut};

use alloc::vec::Vec;

use crate::{api::fs::Block, device::BlockAddr};

use super::{superblock::{self, data_index_to_lba, lba_to_data}, bitmap::Bitmap, inode::{Inode}, index_block::IndexBlock};

static mut ACTIVE_INODE: Option<Inode> = None;


pub(super) const ROOT_DIR_INDEX: u32 = 1;

pub fn set_active_dir(dir: Option<Inode>) {
    unsafe {ACTIVE_INODE = dir}
}

pub fn active_dir<'a>() -> Option<&'a mut Inode> {
    unsafe {ACTIVE_INODE.as_mut()}
}

pub fn allocate_datablock() -> Option<DataBlock> {
    for block_index in 0..superblock::data_size().unwrap() {
        if !Bitmap::is_allocated(block_index) {
            Bitmap::alloc(block_index);
            return DataBlock::read(block_index)
        };
    }

    return None;
}

pub fn free_datablock(index: BlockAddr) {
    if Bitmap::is_allocated(index) {
        Bitmap::free(index)
    }
}

pub fn data_block(index: BlockAddr) -> Option<DataBlock> {
    match superblock::data_block(index) {
        Some(block) => Some(DataBlock { _block: block }),
        None => None
    }
}

pub fn root() -> Option<Inode> {
    Inode::read(ROOT_DIR_INDEX)
}

pub fn create_file(name: &str, size: usize) -> Option<Inode> {
    Inode::new_file(name, size)
}

pub fn allocate_indexblock() -> Option<IndexBlock> {
    IndexBlock::allocate()
}

pub fn free_indexblock(index: BlockAddr) {
    IndexBlock::read(index).unwrap().clear();
}




#[derive(Debug, Clone)]
pub struct DataBlock {
    _block: Block
}

impl DataBlock {

    pub fn allocate() -> Option<Self> {
        allocate_datablock()
    }

    pub fn data_index(&self) -> BlockAddr {
        lba_to_data(self._block.addr())
    }

    pub fn block(&self) -> &Block {
        &self._block
    }

    pub fn block_mut(&mut self) -> &mut Block {
        &mut self._block
    }

    pub fn empty(index: BlockAddr) -> Self {
        Self {
            _block: Block::read(super::superblock::data_index_to_lba(index)).unwrap()
        }
    }

    pub fn read(index: BlockAddr) -> Option<Self> {
        match Block::read(data_index_to_lba(index)) {
            None => None,

            Some(block) => Some(Self {_block: block})
        }
    }

    pub fn write(&mut self) {
        self._block.write()
    }

    pub fn data(&self) -> &[u8] {
        self._block.data()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        self._block.data_mut()
    }
}

impl Index<usize> for DataBlock {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data()[index]
    }
}


impl IndexMut<usize> for DataBlock {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data_mut()[index]
    }
}


impl Into<Vec<u8>> for IndexBlock {
    fn into(self) -> Vec<u8> {
        let mut buffer = Vec::new();
        for block in self.datablocks() {
            for byte in block.data() {
                buffer.push(*byte);
            }
        }
        buffer
    }
}

pub fn file(path: &str) -> &str {
    path.split('/').collect::<Vec<&str>>().last().unwrap()
}