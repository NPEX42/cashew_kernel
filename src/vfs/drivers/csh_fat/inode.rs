use alloc::{string::String, vec::Vec};

use crate::{api::fs::Block, device::BlockAddr, klog, trace_enter, trace_exit};

use super::{index_block::IndexBlock, data_block, DataBlock, superblock::{self, data_index_to_lba}, ROOT_DIR_INDEX, file::File};



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
#[derive(Debug, Clone)]
pub struct Inode {
    _block: Block,
    kind: EntryKind,
    size: u32,
    block_index: IndexBlock,
    name: String,
}

impl Inode {


    fn dir_with_addr(block: BlockAddr, name: &str) -> Self {
        Self {
            _block: Block::read(data_index_to_lba(block)).unwrap(),
            kind: EntryKind::Directory,
            block_index: IndexBlock::allocate().unwrap(),
            size: 0,

            name: name.into()
        }
    }

    pub(crate) fn create_root() -> Inode {
        Self::dir_with_addr(ROOT_DIR_INDEX, "/")
    }

    pub fn new_dir(_name: &str) -> Option<Inode> {
            unimplemented!()
    }

    pub fn new_file(_name: &str, _size: usize) -> Option<Inode> {
        unimplemented!()
    }

    pub fn read(index: u32) -> Option<Inode> {
        if let Some(block) = data_block(index) {
            let kind = EntryKind::from_u8(block[0]);
            let size = block.block().read_u32_be(1);
            let index_block = block.block().read_u32_be(5);
            let name_len = block[10];
            let name = block.block().read_utf8(11, name_len as usize);
            klog!("Index Block DB-{}, LBA-{}\n", index_block, data_index_to_lba(index_block));
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

    pub fn add_child(&mut self, child: Inode) {
        self.block_index.add_inode(&child);
    }

    pub fn add_file(&mut self, child: &File) {
        self.block_index.add_inode(&child.inode);
        self.sync();
    }

    pub fn index(&self) -> BlockAddr {
        superblock::partition_size().unwrap() - self._block.addr()
    }

    pub fn datablocks(&self) -> Vec<DataBlock> {
        self.block_index.datablocks()
    }

    pub fn children(&self) -> Vec<Inode> {
        self.block_index.get_inodes()
    }

    pub fn sync(&mut self) {
        trace_enter!();
        self._block.data_mut()[0] = self.kind as u8;
        self._block.write_u32_be(1, self.size);
        self._block.write_u32_be(5, self.block_index.data_index());
        self._block.data_mut()[10] = self.name.as_bytes().len() as u8;
        self._block.write_utf8(11, &self.name);
        self._block.write();
        trace_exit!();

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

    pub fn data(&self) -> Vec<u8> {
        self.block_index.data(self.size as usize)
    }

    pub fn child(&self, name: &str) -> Option<Inode> {
        let children = self.block_index.get_inodes();
        for child in children {
            if child.name == name {
                return Some(child)
            }
        }


        None
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}
