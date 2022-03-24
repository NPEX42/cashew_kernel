use bit_field::BitField;

use crate::device::BlockAddr;

use super::superblock;

pub struct Bitmap;

impl Bitmap {
    pub fn alloc(index: BlockAddr) {
        let mut block = superblock::bitmap_block(index / 4096).unwrap();
        block.data_mut()[(index / 8) as usize].set_bit((index % 8) as usize, true);
        block.write()
    }

    pub fn free(index: BlockAddr) {
        let mut block = superblock::bitmap_block(index / 4096).unwrap();
        block.data_mut()[(index / 8) as usize].set_bit((index % 8) as usize, false);
        block.write()
    }

    pub fn is_allocated(index: BlockAddr) -> bool {
        let block = superblock::bitmap_block(index / 4096).unwrap();
        block.data()[(index / 8) as usize].get_bit((index % 8) as usize)
    }
}