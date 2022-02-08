use crate::{ata, device::*};


#[derive(Debug)]
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

    pub fn read(addr: BlockAddr) -> Result<Self, ()> {
        Ok(Self {
            addr,
            data: read(addr)?,
        })
    }

    pub fn write(&self) -> Result<(), ()> {
        write(self.addr, &self.data)
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
}

pub struct LinkedBlock {
    block: Block
}

impl TryInto<LinkedBlock> for Block {
    type Error = ();
    fn try_into(self) -> Result<LinkedBlock, Self::Error> {
        LinkedBlock::read(self.addr())
    }
}


impl LinkedBlock {
    pub fn new(addr: BlockAddr) -> Self {
        Self {
            block: Block::empty(addr),
        }
    }

    pub fn read(addr: BlockAddr) -> Result<LinkedBlock, ()> {
        Ok(Self {
            block: Block::read(addr)?
        })
    }

    pub fn write(&self) -> Result<(), ()> {
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
}

