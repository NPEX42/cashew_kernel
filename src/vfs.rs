use alloc::{vec::Vec};

use crate::device::BlockAddr;

pub mod block;
pub mod drivers;



pub trait FileSystem {
    fn root<'a>(&self) -> &'a dyn DirIO;


}

pub trait FileIO {
    fn name(&self) -> &str;
    fn size(&self) -> &str;

    fn write(&mut self, data: &[u8]);
    fn append(&mut self, data: &[u8]);
    fn read(&mut self, buffer: &mut [u8]) -> usize;
    fn read_all(&mut self) -> Vec<u8>;

    fn close(&mut self);
    fn erase(&mut self);

    fn head(&mut self) -> BlockAddr;
    fn set_head(&mut self, addr: BlockAddr);
}

pub trait DirIO {
    fn create_file(&mut self, name: &str);
    fn open_file(&mut self, name: &str, mode: u8) -> bool;
    fn delete_file(&mut self, name: &str) -> bool;

    fn name(&self) -> &str;
    fn size(&self) -> usize;
    fn erase(&mut self);
    fn erase_rec(&mut self);
}