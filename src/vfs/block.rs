use core::{fmt::Display, mem::size_of};

use alloc::{vec::Vec, string::String};

use crate::{
    ata::{self},
    device::{self, *},
    klog,
};

#[derive(Debug, Clone, Copy)]
pub struct Block {
    addr: BlockAddr,
    data: [u8; ata::BLOCK_SIZE],
}

impl Display for Block {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#}", self.addr)
    }
}

impl Block {

    pub fn read(addr: u32) -> Option<Block> {
        match device::read(addr) {
            Ok(data) => Some(Self { addr, data }),
            Err(_) => {
                klog!("Failed To Read Block {:#X}\n", addr);
                None
            }
        }
    }

    pub fn write(&self) {
        match device::write(self.addr, &self.data) {
            Ok(_) => {}
            Err(_) => {
                klog!("Failed To Write Block {:#X}\n", self.addr);
            }
        }
    }

    pub fn empty(addr: BlockAddr) -> Self {
        Self {
            addr,
            data: [0; ata::BLOCK_SIZE],
        }
    }
    
    #[deprecated]
    pub fn allocate() -> Option<Block> {
        unimplemented!()
    }

    #[deprecated]
    pub fn free(&mut self) {
        unimplemented!()
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
        Self { addr, data }
    }

    pub fn read_u16_be(&self, index: usize) -> u16 {
        assert!(index + 1 < self.data.len());
        u16::from_be_bytes(self.data[index..index + 2].try_into().unwrap())
    }

    pub fn read_u32_be(&self, index: usize) -> u32 {
        klog!("Index: {}\n", index);
        assert!(index + 3 < self.data.len(), "Index: {}", index);
        u32::from_be_bytes(self.data[index..index + 4].try_into().unwrap())
    }

    pub fn read_u64_be(&self, index: usize) -> u64 {
        assert!(index + 7 < self.data.len());
        u64::from_be_bytes(self.data[index..index + 8].try_into().unwrap())
    }

    pub fn write_u16_be(&mut self, index: usize, value: u16) {
        assert!(index + size_of::<u16>() < self.data.len());
        self.data[index..index + size_of::<u16>()].copy_from_slice(&value.to_be_bytes());
    }

    pub fn write_u32_be(&mut self, index: usize, value: u32) {
        assert!(index + size_of::<u32>() < self.data.len());
        self.data[index..index + size_of::<u32>()].copy_from_slice(&value.to_be_bytes());
    }

    pub fn write_u64_be(&mut self, index: usize, value: u64) {
        assert!(index + size_of::<u64>() < self.data.len());
        self.data[index..index + size_of::<u64>()].copy_from_slice(&value.to_be_bytes());
    }

    pub fn read_u8_slice(&self, index: usize, data: &mut [u8]) {
        assert!(index + data.len() < self.data.len());
        data.copy_from_slice(&self.data[index..index + data.len()])
    }

    pub fn write_u8_slice(&mut self, index: usize, data: &[u8]) {
        assert!(index + data.len() < self.data.len());
        self.data[index..index + data.len()].copy_from_slice(data);
    }

    pub fn read_utf8z(&self, offset: usize) -> String {
        let mut buffer = Vec::new();
        for i in offset..self.data().len() {
            if self.data[i] > 0 {
                buffer.push(self.data[i]);
            } else {break;}
        }

        String::from_utf8(buffer).expect("Failed To Read UTF-8Z")
    }

    pub fn read_utf8(&self, offset: usize, size: usize) -> String {
        let mut buffer = Vec::new();
        let end = self.data.len().min(offset + size);
        for i in offset..end {
            if self.data[i] > 0 {
                buffer.push(self.data[i]);
            } else {break;}
        }

        String::from_utf8(buffer).expect("Failed To Read UTF-8")
    }

    pub fn write_utf8(&mut self, offset: usize, text: &str) {
        self.write_u8_slice(offset, text.as_bytes());
    }

    pub fn write_block_addr_be(&mut self, index: usize, addr: BlockAddr) {
        let size = size_of::<BlockAddr>();
        self.data[index..index+size].copy_from_slice(&addr.to_be_bytes())
    }
}