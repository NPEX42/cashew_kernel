

use alloc::{boxed::Box, string::String, vec::Vec};


pub mod ustar;
pub mod simple_fat;

pub mod disk_map;


pub trait VirtFileSystem {
    fn open_file(&self, filename: &str) -> Option<Box<dyn FileIO>>;
}

pub trait FileRead {
    fn read(&self, index: usize) -> u8;

    fn read_to_string(&self) -> String;
    fn read_to_vec(&self) -> Vec<u8>;
    fn read_bytes(&self, buffer: &mut [u8]) -> usize;
}

pub trait FileWrite {
    fn write(&mut self, index: usize, value: u8);
}

pub trait FileAppend {
    fn append(&mut self, value: u8);
    fn append_vec(&mut self, other: Vec<u8>);
    fn append_bytes(&mut self, bytes: &[u8]);
}

pub trait FileIO: FileWrite + FileRead + FileAppend {
    fn close(&mut self);
    fn size(&self) -> usize;
    fn rename(&mut self, name: &str);
}