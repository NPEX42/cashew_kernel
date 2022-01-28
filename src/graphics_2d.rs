use font8x8::UnicodeFonts;

use crate::{vga, fonts, arch, pit};

const HEIGHT: usize = 480;
const WIDTH: usize = 640;
const PIXEL_STRIDE: usize = 4;

static mut VBLANK: bool = false;

pub fn vblank() {
    unsafe {VBLANK = true;}
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
            red:   ((c & 0xFF0000) >> 16) as u8,
            green: ((c & 0x00FF00) >> 8 ) as u8,
            blue:  ((c & 0x0000FF) >> 0 ) as u8,
            alpha: 255
        }
    }
}

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
                self.write_pixel(x, y, Pixel::hex(0x000000));
            }
        }
    }
    
    pub fn clear(&mut self, pixel: Pixel) {
        self.pixels.fill(pixel);
    }

    pub const fn new() -> Frame {
        Frame {
            pixels: [Pixel::hex(0xFFFFFF); HEIGHT * WIDTH],
        }
    }
    #[inline(always)]
    pub fn swap(&self) {
        
        pit::sync();

        //crate::sprint!("Swapping Frame Buffer.\n");

        vga::set_pixels(&self.pixels);
        unsafe {VBLANK = false;}
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

    pub fn draw_char(&mut self, 
        x: usize, y: usize,
        chr: char,
        fg: Pixel, bg: Pixel )
    {
        let font = fonts::BASIC_FONTS.get(chr).unwrap_or(fonts::UNICODE_REPLACEMENT);

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