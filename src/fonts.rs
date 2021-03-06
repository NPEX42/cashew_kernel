pub use font8x8::{BASIC_FONTS, BLOCK_FONTS, BOX_FONTS, GREEK_FONTS, HIRAGANA_FONTS, LATIN_FONTS};

pub const FONT_HEIGHT: usize = 8;
pub const FONT_WIDTH: usize = 8;

pub const UNICODE_REPLACEMENT: [u8; FONT_HEIGHT] = [
    0b00011000, 0b00111100, 0b01100110, 0b11110111, 0b11101111, 0b01111110, 0b00101100, 0b00011000,
];

