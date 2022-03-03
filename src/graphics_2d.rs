pub mod palletes;

use font8x8::UnicodeFonts;
use core::ops::*;
use crate::{fonts, pit, vga};

const HEIGHT: usize = 480;
const WIDTH: usize = 640;

static mut VBLANK: bool = false;

pub fn vblank() {
    unsafe {
        VBLANK = true;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Pixel {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub alpha: u8,
}

impl Pixel {
    pub const fn hex(c: u32) -> Pixel {
        Self {
            red: ((c & 0xFF0000) >> 16) as u8,
            green: ((c & 0x00FF00) >> 8) as u8,
            blue: ((c & 0x0000FF) >> 0) as u8,
            alpha: 255,
        }
    }

    pub const fn argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self {
            alpha: a,
            blue: b,
            green: g,
            red: r
        }
    }

    pub fn as_u32(&self) -> u32 {
        let r = (self.red as u32) << 16;
        let g = (self.green as u32) << 8;
        let b = (self.blue as u32) << 0;

        r | g | b
    } 
}

#[repr(transparent)]
pub struct Frame {
    pixels: [Pixel; HEIGHT * WIDTH],
}

impl Frame {
    pub fn shift_up(&mut self, amount: usize) {
        for y in amount..HEIGHT {
            for x in 0..WIDTH {
                let pixel = self.read_pixel(x, y);
                self.write_pixel(x, y - amount, pixel);
            }
        }

        for y in HEIGHT - amount..HEIGHT {
            for x in 0..WIDTH {
                self.write_pixel(x, y, Pixel::hex(0x0000FF));
            }
        }
    }

    pub fn clear(&mut self, pixel: Pixel) {
        self.pixels.fill(pixel);
    }

    pub const fn new() -> Frame {
        Frame {
            pixels: [Pixel::hex(0x0000FF); HEIGHT * WIDTH],
        }
    }
    #[inline(always)]
    pub fn swap(&self) {
        pit::sync();

        //crate::sprint!("[FrameBuffer]: Swapping Frame Buffer.\n");

        vga::set_pixels(&self.pixels);
        unsafe {
            VBLANK = false;
        }
    }

    #[inline(always)]
    pub fn read_pixel(&self, x: usize, y: usize) -> Pixel {
        let bytes = self.raw_data(x, y);
        bytes
    }

    #[inline(always)]
    pub fn write_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        let offset = x + (WIDTH) * y;
        self.pixels[offset + 0] = pixel;
    }
    #[inline(always)]
    fn raw_data(&self, x: usize, y: usize) -> Pixel {
        let offset = x + (WIDTH) * y;
        self.pixels[offset]
    }

    pub fn blit(&mut self, x: usize, y: usize, w: usize, h: usize, pixels: &[Pixel]) {
        for y_offset in (0..h).step_by(w) {
            for x_offset in 0..w {
                self.write_pixel(x, y, pixels[x_offset + y_offset]);
            }
        }
    }

    pub fn draw_char(&mut self, x: usize, y: usize, chr: char, fg: Pixel, bg: Pixel) {
        let font = if chr >= ' ' {
            fonts::BASIC_FONTS
                .get(chr)
                .unwrap_or(fonts::UNICODE_REPLACEMENT)
        } else if chr == '\n' {
            fonts::NEW_LINE_PRINTABLE
        } else if chr == '\0' {
            fonts::NULL_PRINTABLE
        } else if chr == '\r' {
            fonts::CR_PRINTABLE
        } else {
            fonts::UNICODE_REPLACEMENT
        };

        for row in 0..font.len() {
            for bit in 0..8 {
                if (1 << bit) & font[row] > 0 {
                    self.write_pixel(x + bit, y + row, fg);
                } else {
                    self.write_pixel(x + bit, y + row, bg);
                }
            }
        }
    }
}


impl BitOr for Pixel {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::hex(self.as_u32() | rhs.as_u32())
    }
}

impl BitAnd for Pixel {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::hex(self.as_u32() & rhs.as_u32())
    }
}

impl Div<u8> for Pixel {
    type Output = Self;

    fn div(self, rhs: u8) -> Self::Output {
        let r = self.red / rhs;
        let g = self.green / rhs;
        let b = self.blue / rhs;
        let a = self.alpha / rhs;

        Pixel::argb(a, r, g, b)
    }
}