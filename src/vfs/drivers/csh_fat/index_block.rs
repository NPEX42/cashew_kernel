use core::mem::size_of;

use alloc::vec::Vec;

use crate::{api::fs::Block, device::BlockAddr};

use super::{DataBlock, data_block, allocate_datablock, bitmap::Bitmap};

pub struct IndexBlock {
    _block: Block,
    count: u16,
    blocks: Vec<BlockAddr>
}
#[allow(unused)]
impl IndexBlock {
    pub fn addr(&self) -> u32 {
        self._block.addr()
    }

    pub fn count(&self) -> usize {
        self.count as usize
    }

    pub fn read(index: BlockAddr) -> Option<IndexBlock> {
        if let Some(blk) = DataBlock::read(index) {
            let count = blk.block().read_u16_be(0);
            let mut buffer = Vec::new();
            for i in (4..(count * 4)).step_by(4) {
                buffer.push(blk.block().read_u32_be(i as usize));
            }

            Some(Self {
                _block: *blk.block(),
                blocks: buffer,
                count
            })

        } else {
            None
        }
    }

    pub fn allocate() -> Option<IndexBlock> {
        match Block::allocate() {
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

        self.count = 0;
    }

    pub fn datablocks(&self) -> Vec<DataBlock> {
        let mut buffer = Vec::new();
        for index in &self.blocks {
            if let Some(block) = data_block(*index) {
                buffer.push(block)
            }
        }

        buffer
    }

    pub fn sync(&mut self) {
        self._block.write_u16_be(0, self.blocks.len() as u16);
        let mut idx = 0;
        for offset in (4..(self.blocks.len() * size_of::<u32>())).step_by(size_of::<u32>()) {
            self._block.write_block_addr_be(offset, self.blocks[idx]);
            idx += 1; 
        }
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
                    self.blocks.push(blk.block().addr());
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
                    blk.block_mut().data_mut()[..chunk.len()].copy_from_slice(chunk);
                    blk.write();

                    self.add_block(blk);
                },

                None => break
            }
        }
    }

    fn add_block(&mut self, blk: DataBlock) {
        self.blocks.push(blk.block().addr());
    }
}