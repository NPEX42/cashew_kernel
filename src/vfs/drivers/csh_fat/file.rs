use alloc::vec::Vec;

use crate::vfs::{OpenFlags, CREATE};

use super::{inode::Inode, active_dir, file, create_file};

#[allow(dead_code)]
pub struct File {
    mode: OpenFlags,
    pub(super) inode: Inode,
    pos: usize,
    data: Vec<u8>
}


impl File {
    pub fn open(path: &str, flags: OpenFlags) -> Option<File> {
        if active_dir().is_none() {return None};
        match active_dir().unwrap().child(file(path)) {
            Some(file) => Some(Self::from_inode(file, flags)),
            None => if (flags & CREATE) > 0 {
                Self::create(path, flags)
            } else {
                None
            }
        }
    }

    pub fn create(path: &str, flags: OpenFlags) -> Option<File> {
        match create_file(file(path), 0) {
            Some(node) => Some(Self::from_inode(node, flags)),
            None => None
        }
    }

    fn from_inode(node: Inode, flags: OpenFlags) -> File {
        File { 
            inode: node.clone(), 
            pos: 0, 
            data: node.data().clone(),
            mode: flags
        }
    }
    
    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> usize {
        for i in 0..buffer.len() {
            buffer[i] = self.data[self.pos];
            self.pos += 1;

            if self.pos >= self.data.len() {
                return i;
            }
        }

        return buffer.len();
    }

    pub fn write(&mut self, buffer: &[u8]) -> usize {
        for i in 0..buffer.len() {
            self.data[self.pos] = buffer[i];
            self.pos += 1;

            if self.pos >= self.data.len() {
                return i;
            }
        }

        return buffer.len();
    }

    pub fn close(&mut self) {
        self.inode.set_data(&self.data);
        self.inode.sync()
    }

    pub fn append(&mut self, data: &[u8]) {
        self.data.append(&mut Vec::from(data));
    }
}