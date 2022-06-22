pub type PhysicalBlockAddr = u32;
pub type VirtualBlockAddr = u32;
pub mod fat;
use alloc::{vec::Vec, string::String};
use lazy_static::lazy_static;
use fat::FileAttributeTable;
use crate::{locked::Locked};

use self::fat::FileEntry;

use super::{FileIO, FileWrite, FileRead, FileAppend};
lazy_static! {
    static ref FILE_SYSTEM: Locked<fat::FileAttributeTable> = Locked::new(FileAttributeTable::load(0, 4));
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct File {
    entry: FileEntry,
    data: Vec<u8>,
    pos: usize,
}

pub fn delete_file(name: &str) -> bool {
    let mut fat = FILE_SYSTEM.lock();
    match fat.search_for_file_mut(name) {
        None => false,
        Some(entry) => {
            entry.erase();
            true
        }
    }
}

pub fn load_file(name: &str) -> Option<File> {
    let fat = FILE_SYSTEM.lock();
    
    match fat.search_for_file(name) {
        None => None,
        Some(entry) => {
            Some(File {
                data: entry.to_vec(),
                entry: entry.clone(),
                pos: 0
            })
        }
    }
}

pub fn create_file(name: &str) -> Option<File> {
    let mut fat = FILE_SYSTEM.lock();
    match fat.next_free_entry() {
        None => None,
        Some(mut entry) => {
            entry.set_name(name);
            entry.mark_used();

            fat[entry.index()] = entry.clone();

            Some(File {
                data: Vec::new(),
                entry: entry.clone(),
                pos: 0
            })
        }
    }
}

pub fn list() -> Vec<File> {
    let fat = FILE_SYSTEM.lock();
    let mut files = Vec::new();
    for idx in 0..fat.entry_count() {
        let entry = &fat[idx];
        if entry.is_empty() {continue;}

        files.push(File {data: Vec::new(), entry: entry.clone(), pos: 0});
    }

    files.clone()
}

impl FileWrite for File {
    fn write(&mut self, index: usize, value: u8) {
        self.data[index] = value;
    }
}

impl FileRead for File {
    fn read(&self, index: usize) -> u8 {
        self.data[index]
    }

    fn read_bytes(&self, _: &mut [u8]) -> usize {
        0
    }

    fn read_to_string(&self) -> String {
        String::from_utf8_lossy(&self.data).into()
    }

    fn read_to_vec(&self) -> Vec<u8> {
        self.data.clone()
    }
}

impl FileAppend for File {
    fn append(&mut self, value: u8) {
        self.data.push(value)
    }

    fn append_bytes(&mut self, bytes: &[u8]) {
        self.data.append(&mut bytes.to_vec());
    }


    fn append_vec(&mut self, other: Vec<u8>) {
        self.data.append(&mut other.clone())
    }
}

impl FileIO for File {
    fn close(&mut self) {
        self.entry.set_data(&self.data)
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn rename(&mut self, name: &str) {
        self.entry.set_name(name);
    }

    
}

impl File {
    pub fn name(&self) -> &String {
        self.entry.name()
    }

    pub fn size_on_disk(&self) -> usize {
        self.entry.size()
    }
}