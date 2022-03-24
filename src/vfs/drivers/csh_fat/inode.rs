use alloc::{string::String, vec::Vec};

use crate::api::fs::Block;

use super::{index_block::IndexBlock, data_block, DataBlock};



#[derive(Debug, Clone, Copy, Default)]
pub enum EntryKind {
    File        = 1 << 0,
    Directory   = 1 << 1,
    CharDevice  = 1 << 2,
    BlockDevice = 1 << 3,



    #[default]
    Unknown = 255
}

impl EntryKind {
    pub fn from_u8(x: u8) -> Self {
        match x {
            1 => Self::File,
            2 => Self::Directory,
            4 => Self::CharDevice,
            8 => Self::BlockDevice,
            _ => Self::Unknown,
        }
    }
}

pub struct Inode {
    _block: Block,
    kind: EntryKind,
    size: u32,
    block_index: IndexBlock,
    name: String,
}

impl Inode {
    fn new(name: &str, kind: EntryKind, size: usize, start: IndexBlock) -> Self {
        Self {
            _block: Block::allocate().expect("Failed To Allocate Inode"),
            kind,
            block_index: start,
            size: size as u32,

            name: name.into()
        }
    }

    pub fn new_file(name: &str, size: usize) -> Option<Inode> {
        match IndexBlock::allocate() {
            Some(block) => Some(Self::new(name, EntryKind::File, size, block)),
            None => None
        }
    }

    pub fn read(index: u32) -> Option<Inode> {
        if let Some(block) = data_block(index) {
            let kind = EntryKind::from_u8(block[0]);
            let size = block.block().read_u32_be(1);
            let index_block = block.block().read_u32_be(5);
            let name_len = block[10];
            let name = block.block().read_utf8(11, name_len as usize);

            Some(Self {
                _block: *block.block(),
                block_index: IndexBlock::read(index_block).expect("Failed To Read Index Block"),
                kind,
                name,
                size
            })
        } else {
            None
        }
    }

    pub fn datablocks(&self) -> Vec<DataBlock> {
        self.block_index.datablocks()
    }

    pub fn sync(&mut self) {
        self._block.data_mut()[0] = self.kind as u8;
        self._block.write_u32_be(1, self.size);
        self._block.write_u32_be(5, self.block_index.addr());
        self._block.data_mut()[10] = self.name.as_bytes().len() as u8;
        self._block.write_utf8(11, &self.name);
        self._block.write();


        self.block_index.sync();
    }

    pub fn resize(&mut self, new_size: usize) {
        self.size = new_size as u32;
        self.block_index.resize(new_size as u16);
    }

    pub fn set_data(&mut self, data: &[u8]) {
        self.resize(data.len());
        self.block_index.set_data(data);
    }
}