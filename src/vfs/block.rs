use core::fmt::Display;

use alloc::vec::Vec;

use crate::{ata, device::{self, *}, klog, mem::allocator::BitmapAllocator};

use super::{BlockAllocator, drivers::csh_fat::BlockBitmap};

const DATA_OFFSET: usize = 4;
const DATA_SIZE: usize = 512 - DATA_OFFSET;
#[derive(Debug, Clone, Copy)]
pub struct Block {
    addr: BlockAddr,
    data: [u8; ata::BLOCK_SIZE]
}

impl Block {


    pub fn empty(addr: BlockAddr) -> Self {
        Self {
            addr,
            data: [0; ata::BLOCK_SIZE]
        }
    }

    pub fn allocate() -> Option<Block> {
        BlockBitmap::get().allocate()
    }

    pub fn free(&mut self) {
        BlockBitmap::get().free(self.addr)
    }


    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn addr(&self) -> BlockAddr {
        self.addr
    }

    pub fn from(addr: BlockAddr, data: [u8; 512]) -> Self {
        Self {
            addr,
            data
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LinkedBlock {
    block: Block
}

impl TryInto<LinkedBlock> for Block {
    type Error = ();
    fn try_into(self) -> Result<LinkedBlock, Self::Error> {
        LinkedBlock::read(self.addr())
    }
}



impl Into<Block> for LinkedBlock {
    fn into(self) -> Block {
        self.block
    }
}

#[deprecated]
impl LinkedBlock {
    pub fn new(addr: BlockAddr) -> Self {
        Self {
            block: Block::empty(addr),
        }
    }

    pub fn read(addr: BlockAddr) -> Result<LinkedBlock, ()> {
        Ok(Self {
            block: Block::read(addr).expect("Failed To  Read Block")
        })
    }

    pub fn write(&self) {
        self.block.write()
    }

    pub fn next(&self) -> Option<LinkedBlock> {
        let addr: u32 = u32::from_be_bytes(self.block.data()[0..4].try_into().unwrap());
        if addr == 0 {return None;}
        if let Ok(block) = Self::read(addr) {
            Some(block)
        } else {
            None
        }
    }

    pub fn set_next_addr(&mut self, addr: BlockAddr) {
        let bytes = addr.to_be_bytes();
        self.block.data_mut()[0..4].copy_from_slice(&bytes);
    }

    pub fn set_next(&mut self, next: Block) {
        self.set_next_addr(next.addr())
    }

    pub fn to_vec_sized(&mut self, size: usize) -> Vec<u8> {
        let mut buf = Vec::new();
        for i in 4..size.min(512) {
            buf.push(self.block.data()[i]);
        }

        if size >= 512 {
            if let Some(mut next) = self.next() {
                buf.append(&mut next.to_vec_sized(size - 512));
            }
        }

        buf
    }

    fn to_vec_u32(&self) -> Vec<u32>  {
        let mut blocks = Vec::new();

        if let Some(next) = self.next() {
            blocks.append(&mut next.to_vec_u32());
        }

        blocks
    }

    pub fn clear(&mut self, mut allocator: impl BlockAllocator) -> Result<(),()> {
        let blocks = self.to_vec_u32();
        klog!("Found {} Blocks ({:?})\n", blocks.len(), blocks);
        for block in blocks {
            allocator.free(block);
            let mut b = Block::read(block).expect("Failed To Read Block");
            b.data.fill(0);
            b.write();
        }
        Ok(())
    }

    pub fn addr(&self) -> u32 {
        self.block.addr
    }


    pub fn set_data(&mut self, data: &[u8]) -> Result<(), ()> {
        let mut blocks = self.blocks();
        for (index, chunk) in data.chunks(DATA_SIZE).enumerate() {
            blocks[index].data_mut()[DATA_OFFSET..(DATA_OFFSET+chunk.len())].copy_from_slice(chunk);
            blocks[index].write();
        }

        self.write();
        
        Ok(())
    }

    pub fn blocks(&self) -> Vec<Block> {
        let mut buf = Vec::new();
        buf.push(self.block);
        if let Some(next) = self.next() {
            buf.append(&mut next.blocks());
        }

        buf
    }



}

impl Display for LinkedBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{:08x}]", self.addr())?;
        if let Some(next) = self.next() {
            write!(f, "--> {:}", next)?;
        }

        Ok(())
    }
}









impl Block {
    pub fn read(addr: u32) -> Option<Block> {
        match device::read(addr) {
            Ok(data) => Some(Self {addr, data}),
            Err(_) => {klog!("Failed To Read Block {:#X}\n", addr); None},
        }
    }

    pub fn write(&self)  {
        match device::write(self.addr, &self.data) {
            Ok(_)  => {},
            Err(_) => { klog!("Failed To Write Block {:#X}\n", self.addr);} 
        }
    }

    

}

