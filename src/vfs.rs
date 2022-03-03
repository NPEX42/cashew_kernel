use alloc::{vec::Vec};

use crate::device::BlockAddr;

pub mod block;
pub mod drivers;

pub type FileDesc = u16;

pub const FD_STDOUT: FileDesc = 1;
pub const FD_STDIN: FileDesc = 0;
pub const FD_STDERR: FileDesc = 2;

pub enum SeekPos {
    FromStart(usize),
    FromEnd(usize),
    FromCurrent(usize),
    None
}

pub trait VirtualFileSystem {
    fn open(name: &str, flags: &str) -> Option<FileDesc>;
    fn read(fd: FileDesc, buffer: &mut [u8]) -> usize;
    fn write(fd: FileDesc, buffer: &[u8]) -> usize;
    fn seek(fd: FileDesc, pos: SeekPos) -> usize;
    fn close(fd: FileDesc) -> Result<(), ()>;
}