use core::mem::size_of;
use crate::{klog, kerr, trace_enter, trace_exit};

use alloc::vec::Vec;

use crate::{device::BlockAddr};

use super::{DataBlock, data_block, allocate_datablock, bitmap::Bitmap, inode::Inode};

#[derive(Debug, Clone)]
pub struct IndexBlock {
    _block: DataBlock,
    count: u16,
    blocks: Vec<BlockAddr>
}
#[allow(unused)]
impl IndexBlock {
    pub fn lba(&self) -> u32 {
        self._block.block().addr()
    }

    pub fn data_index(&self) -> u32 {
        self._block.data_index()
    }



    pub fn count(&self) -> usize {
        self.count as usize
    }

    pub fn read(index: BlockAddr) -> Option<IndexBlock> {
        if let Some(blk) = DataBlock::read(index) {
            let count = blk.block().read_u16_be(0);
            let mut buffer = Vec::new();
            klog!("Index Block Count: {}\n", count);
            for i in (4..(count * 4)).step_by(4) {
                buffer.push(blk.block().read_u32_be(i as usize));
            }

            klog!("Blocks: {:#?}\n", buffer);

            Some(Self {
                _block: blk,
                blocks: buffer,
                count
            })

        } else {
            None
        }
    }

    pub fn allocate() -> Option<IndexBlock> {
        match DataBlock::allocate() {
            Some(block) => Some(Self {
                _block: block,
                count: 0, 
                blocks: Vec::new(),
            }),

            None => None,
        }
    }

    pub fn clear(&mut self) {
        for data_block in self.datablocks() {
            Bitmap::free(data_block.block().addr());
        }

        self.blocks.clear();
        self.count = 0;
        self.sync();
    }

    pub fn datablocks(&self) -> Vec<DataBlock> {
        let mut buffer = Vec::new();
        for index in &self.blocks {
            if let Some(block) = data_block(*index) {
                buffer.push(block)
            } else {
                klog!("Error Loading Datablock {}", index);
            }
        }

        buffer
    }

    pub fn data(&self, len: usize) -> Vec<u8> {
        let mut buffer = Vec::new();

        for block in self.datablocks() {
            for byte in block.data() {
                buffer.push(*byte);

                if buffer.len() > len {
                    break;
                }
            }
        }

        buffer
    }

    pub fn sync(&mut self) {
        trace_enter!();
        klog!("Vec Blocks: {}, Count: {}\n", self.blocks.len(), self.count());
        //assert!(self.blocks.len() == self.count as usize);
        self._block.block_mut().write_u16_be(0, self.count);
        let mut idx = 0;
        let entry_size = size_of::<u32>();
        let end = self.count as usize * entry_size;
        for offset in (entry_size..end).step_by(entry_size) {
            self._block.block_mut().write_block_addr_be(
                offset, 
                self.blocks[idx]
            );
            idx += 1; 
        }
        self._block.block_mut().write();
        trace_exit!();
    }

    pub fn resize(&mut self, new_size: u16) {
        
        let mut blocks = new_size / 512;
        if new_size % 512 > 0 {
            blocks += 1;
        }

        let new_size = blocks;

        if self.count < new_size {
            self.blocks.truncate(new_size as usize)
        } else if self.count > new_size {
            for _ in self.count..new_size {
                if let Some(blk) = allocate_datablock() {
                    self.blocks.push(blk.data_index());
                }
            }
        }

        self.count = new_size;

        self.sync();
    }

    pub fn set_data(&mut self, data: &[u8]) {
        self.clear();
        for chunk in data.chunks(512) {
            match DataBlock::allocate() {
                Some(mut blk) => {
                    //klog!("Allocated Datablock {:?}", blk);
                    blk.block_mut().data_mut()[..chunk.len()].copy_from_slice(chunk);
                    blk.write();

                    self.add_datablock(blk);
                },

                None => break
            }
        }
    }

    pub fn set_inodes(&mut self, inodes: Vec<Inode>) {

        klog!("Setting Inodes\n");

        self.clear();
        for inode in inodes {
            self.add_inode(&inode);
        }
    }

    pub fn get_inodes(&self) -> Vec<Inode> {
        let mut inodes = Vec::new();

        for index in &self.blocks {
            match Inode::read(*index) {
                Some(inode) => inodes.push(inode),
                None => kerr!("Failed To Read Inode {}", index),
            }
        }

        inodes
    }

    fn add_datablock(&mut self, blk: DataBlock) {
        self.blocks.push(blk.data_index());
        self.count += 1;
        self.sync();
    }

    pub fn add_inode(&mut self, blk: &Inode) {
        klog!("Adding Inode: '{}'\n", blk.name());
        self.blocks.push(blk.index());
        self.count += 1;
        self.sync();


        klog!("Block Count: {}\n", self.count);
    }
}