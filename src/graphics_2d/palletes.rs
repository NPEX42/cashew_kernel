
use conquer_once::spin::OnceCell;

use super::Pixel;

pub const RED  : Pixel = Pixel::hex(0xFF0000);
pub const GREEN: Pixel = Pixel::hex(0x00FF00);
pub const BLUE : Pixel = Pixel::hex(0x0000FF);
pub const BLACK: Pixel = Pixel::hex(0x000000);
pub const WHITE: Pixel = Pixel::hex(0xFFFFFF);

pub const MONOCHROME: [Pixel; 2] = [
    BLACK,
    WHITE
];

static CGA: OnceCell<[Pixel; 16]> = OnceCell::uninit();

pub fn cga<'a>() -> &'a [Pixel; 16] {
    CGA.get_or_init(|| {[(BLACK) / 2,
        (BLUE) / 2,
        (GREEN) / 2,
        (GREEN | BLUE) / 2,
        (RED) / 2,
        (RED | BLUE) / 2,
        (RED | GREEN) / 2,
        (WHITE) / 2,
    
        (BLACK) / 1,
        (BLUE) / 1,
        (GREEN) / 1,
        (GREEN | BLUE) / 1,
        (RED) / 1,
        (RED | BLUE) / 1,
        (RED | GREEN) / 1,
        (WHITE) / 1,]})
}

