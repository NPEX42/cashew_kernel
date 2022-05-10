

use alloc::{boxed::Box, string::String, vec::Vec};


pub mod ustar;
pub mod simple_fat;

pub mod disk_map;


pub trait VirtFileSystem {
    fn open_file(filename: &str) -> Option<Box<dyn FileIO>>;
}

pub trait FileRead {
    fn read(&self, index: usize) -> u8;
}

pub trait FileWrite {
    fn write(&self, index: usize, value: u8);
}

pub trait FileIO: FileWrite + FileRead {
    fn close(&mut self);
    fn read_to_string(&self) -> String;
    fn read_to_vec(&self) -> Vec<u8>;
    fn read_bytes(&self, buffer: &mut [u8]) -> usize; 
}