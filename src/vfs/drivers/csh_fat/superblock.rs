use bit_field::BitField;

use crate::{
    csh::{ExitCode, ShellArgs},
    device::{self},
    input, print, println,
    vfs::block::Block, terminal, graphics_2d::{Pixel, ProgressBar},
};
const SUPERBLOCK_ADDR: u32 = 0;
const SIGNATURE: &[u8; 3] = b"CFS";

#[repr(usize)]
pub enum FieldOffset {
    Signature,    // [u8; 3]
    VerMaj = 3,   // u8
    VerMin = 4,   // u8
    PartSize = 5, // u32
    FatStart = 9, // u32
    FatSize = 13, // u32
    DataEnd = 14, // u32

    CheckSum = 511, // u8
}

impl Into<usize> for FieldOffset {
    fn into(self) -> usize {
        self as usize
    }
}

pub fn validate() -> bool {
    if let Some(b) = Block::read(SUPERBLOCK_ADDR) {
        let mut buffer = [0; 3];
        b.read_u8_slice(FieldOffset::Signature.into(), &mut buffer);
        let mut chk = 0;
        for byte in 0..FieldOffset::CheckSum.into() {
            chk += b.data()[byte] as u32;
        }
        let chk: u8 = (chk & 0xFF) as u8;

        return buffer.eq(SIGNATURE) && (chk == checksum());
    } else {
        return false;
    }
}

pub fn format(v_maj: u8, v_minor: u8, part_size: u32, data_size: u32) {
    if let Some(mut b) = Block::read(SUPERBLOCK_ADDR) {
        b.write_u8_slice(0, SIGNATURE);
        b.data_mut()[4] = v_minor;
        b.data_mut()[3] = v_maj;
        b.write_u32_be(FieldOffset::PartSize.into(), part_size);
        b.write_u32_be(FieldOffset::DataEnd.into(), part_size - data_size);
        b.write_u32_be(FieldOffset::FatSize.into(), 256);

        let mut checksum: u32 = 0;
        for byte in 0..FieldOffset::CheckSum.into() {
            checksum += b.data()[byte] as u32;
        }

        b.data_mut()[FieldOffset::CheckSum as usize] = (checksum & 0xFF) as u8;

        b.write();
    }
}

pub fn version_minor() -> u8 {
    if let Some(b) = Block::read(SUPERBLOCK_ADDR) {
        b.data()[FieldOffset::VerMin as usize]
    } else {
        0
    }
}

pub fn checksum() -> u8 {
    if let Some(b) = Block::read(SUPERBLOCK_ADDR) {
        b.data()[FieldOffset::CheckSum as usize]
    } else {
        0
    }
}

pub fn version_major() -> u8 {
    if let Some(b) = Block::read(SUPERBLOCK_ADDR) {
        b.data()[FieldOffset::VerMaj as usize]
    } else {
        0
    }
}

pub fn partition_size() -> Option<u32> {
    if let Some(b) = Block::read(SUPERBLOCK_ADDR) {
        Some(b.read_u32_be(FieldOffset::PartSize.into()))
    } else {
        None
    }
}

pub fn fat_start() -> Option<u32> {
    if let Some(b) = Block::read(SUPERBLOCK_ADDR) {
        Some(b.read_u32_be(FieldOffset::FatStart.into()))
    } else {
        None
    }
}

pub fn fat_size() -> Option<u32> {
    if let Some(b) = Block::read(SUPERBLOCK_ADDR) {
        Some(b.read_u32_be(FieldOffset::FatSize.into()))
    } else {
        None
    }
}

pub fn fat_end() -> u32 {
    fat_start().unwrap() + fat_size().unwrap()
}

pub fn bitmap_start() -> u32 {
    fat_start().unwrap() + fat_size().unwrap()
}

pub fn bitmap_size() -> u32 {
    data_size().unwrap() / (8 * 512)
}

pub fn bitmap_end() -> u32 {
    bitmap_start() + bitmap_size()
}

pub fn data_size() -> Option<u32> {
    Some(partition_size().unwrap() - data_end().unwrap())
}

pub fn data_end() -> Option<u32> {
    if let Some(b) = Block::read(SUPERBLOCK_ADDR) {
        Some(partition_size().unwrap() - b.read_u32_be(FieldOffset::DataEnd.into()))
    } else {
        None
    }
}

pub fn data_block(index: u32) -> Option<Block> {
    if index > data_size().unwrap() {
        return None;
    };
    let lba = partition_size().unwrap() - index;

    Block::read(lba)
}

pub fn bitmap_block(index: u32) -> Option<Block> {
    Block::read(bitmap_start() + index)
}

pub fn alloc_count() -> usize {
    let mut count = 0;
    for addr in bitmap_start()..bitmap_end() {
        if let Some(block) = Block::read(addr) {
            for byte in block.data() {
                for bit in 0..8 {
                    if byte.get_bit(bit) {
                        count += 1
                    }
                }
            }
        }
    }
    count
}

pub fn preload() {
    let size = bitmap_size() as f32;
    let start = bitmap_start();

    let mut bmp_pb = ProgressBar::new(
        start as f32, 
        bitmap_end() as f32,
        Pixel::hex(0xFFFFFFFF),
        256.0);

    for bmp_addr in start..bitmap_end() {
        print!(
            "Preloading Bitmap - {:02.3}%\r",
            ((bmp_addr - start) as f32 / size) * 100.0
        );
        bmp_pb.update(bmp_addr as f32);
        bmp_pb.draw(2,  terminal::y() + 10);
        

        Block::read(bmp_addr).unwrap();
    }
    print!("Preloading Bitmap - {:02.3}%\r", 100.0);
    println!();
    bmp_pb.draw(2, terminal::y() + 10);
    println!();

    let size = fat_size().unwrap() as f32;

    let mut fat_pb = ProgressBar::new(
        fat_start().unwrap() as f32, 
        fat_end() as f32,
        Pixel::hex(0xFFFFFFFF),
        256.0);

    for fat_addr in fat_start().unwrap()..fat_end() {
        print!(
            "Preloading FAT - {:02.3}%\r",
            (fat_addr as f32 / size) * 100.0
        );
        fat_pb.update(fat_addr as f32);
        fat_pb.draw(2,  terminal::y() + 8);
        Block::read(fat_addr).unwrap();
    }

    

    print!("Preloading FAT - {:02.3}%\r", 100.0);
    println!();
    fat_pb.draw(2,  terminal::y() + 10);
    println!();
}

pub fn csh_format(_args: ShellArgs) -> ExitCode {
    let blocks = device::info().unwrap().blocks;
    let part_size: u32 = input::prompt("Input Partition Size: ").parse().unwrap();
    let part_size = part_size.min(blocks as u32);
    let data_size: u32 = input::prompt("Input Data Size: ").parse().unwrap();
    let erase_drive: bool =
        input::prompt("Erase Drive Beforehand? [y/N]").eq_ignore_ascii_case("y");
    if erase_drive {
        for i in 0..part_size {
            Block::empty(i).write();
            print!("Erasing Drive - {}/{}\r", i, part_size);
        }
    }
    println!();
    print!("Writing Superblock - ");
    format(1, 0, part_size, data_size);
    println!("Done...");
    let bmp_end = bitmap_end();
    for addr in bitmap_start()..bmp_end {
        print!("Clearing Bitmap - {}/{}\r", addr, bmp_end);
        Block::empty(addr).write();
    }

    println!();

    ExitCode::Ok
}
