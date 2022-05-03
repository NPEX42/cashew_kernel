use self::block::Block;

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
    None,
}


pub trait BlockAllocator {
    fn allocate(&mut self) -> Option<Block>;
    fn free(&mut self, block: u32);
}


pub type OpenFlags = u8;
pub const READ: OpenFlags =   1 << 0;
pub const WRITE: OpenFlags =  1 << 1;
pub const APPEND: OpenFlags = 1 << 2;
pub const CREATE: OpenFlags = 1 << 3;

pub struct FileVTable {
    open: fn (path: &str, flags: OpenFlags) -> FileDesc,
    close: fn(FileDesc),
    write: fn(FileDesc, &[u8]) -> usize,
    read: fn (FileDesc, &mut [u8]) -> usize,
    seek: fn (FileDesc, SeekPos),
}

impl FileVTable {
    pub fn open(&self, path: &str, flags: OpenFlags) -> FileDesc {
        (self.open)(path, flags)
    }

    pub fn close(&self, fd: FileDesc) {
        (self.close)(fd)
    }

    pub fn write(&self, fd: FileDesc, data: &[u8]) -> usize {
        (self.write)(fd, data)
    }

    pub fn read(&self, fd: FileDesc, data: &mut [u8]) -> usize {
        (self.read)(fd, data)
    }

    pub fn seek(&self, fd: FileDesc, pos: SeekPos) {
        (self.seek)(fd, pos)
    }
}


