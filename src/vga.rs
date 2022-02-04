use core::{fmt::Display, mem::size_of};

use bootloader::boot_info::FrameBufferInfo;
use conquer_once::spin::OnceCell;
use font8x8::UnicodeFonts;

use crate::{
    fonts,
    graphics_2d::{self, Pixel},
    locked::Locked,
};

const BUFFER_WIDTH: usize = 640;
const BUFFER_HEIGHT: usize = 480;
const BYTES_PER_PIXEL: usize = 4;

pub static BACK_BUFFER: OnceCell<Locked<[u8; BUFFER_WIDTH * BUFFER_HEIGHT * BYTES_PER_PIXEL]>> =
    OnceCell::uninit();

pub static VGA: OnceCell<Locked<Vga<'static>>> = OnceCell::uninit();

pub struct RawImage {
    pixels: [Color24; (640 * 480)],
}

impl RawImage {
    pub fn new() -> Self {
        Self {
            pixels: [Color24::grey(255); 640 * 480],
        }
    }

    pub fn width(&self) -> usize {
        640
    }

    pub fn height(&self) -> usize {
        480
    }

    pub fn pixel(&self, x: usize, y: usize) -> Color24 {
        self.pixels[x + y * 640]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color24) {
        self.pixels[x + y * 640] = color
    }
}

pub fn screen_height() -> usize {
    get().unwrap().lock().screen_height()
}

pub fn screen_width() -> usize {
    get().unwrap().lock().screen_width()
}

pub fn write_pixel(x: usize, y: usize, col: Pixel) {
    get().unwrap().lock().write_pixel(x, y, col)
}

pub fn draw_filled_rect(x: usize, y: usize, h: usize, w: usize, col: Pixel) {
    get().unwrap().lock().draw_filled_square(x, y, w, h, col);
}

pub fn put_char(x: usize, y: usize, chr: char, fg: Pixel, bg: Pixel) {
    get().unwrap().lock().put_char(x, y, chr, (fg, bg));
}

pub fn clear(color: Pixel) {
    get().unwrap().lock().clear(color);
}

pub fn draw_bitmap(x: usize, y: usize, fg: Pixel, bg: Pixel, bitmap: &[u8]) {
    get().unwrap().lock().draw_char_8(x, y, fg, bg, bitmap);
}

pub fn set_pixels(pixels: &[Pixel]) {
    let dest = get().unwrap().lock().framebuffer_mut().as_mut_ptr() as *mut u128;
    let src = pixels.as_ptr() as *const u128;

    unsafe {
        core::ptr::copy(
            src,
            dest,
            pixels.len() / (size_of::<u128>() / size_of::<Pixel>()),
        );
    }
}

pub fn get() -> Option<&'static Locked<Vga<'static>>> {
    VGA.get()
}

pub fn initialize(frame_buffer: *mut u8, info: FrameBufferInfo) {
    let frame_buffer = frame_buffer as *mut Pixel;
    VGA.get_or_init(|| {
        Locked::new(Vga::from_framebuffer(
            unsafe { core::slice::from_raw_parts_mut(frame_buffer, 640 * 480) },
            info,
        ))
    });
}

pub struct Vga<'fb> {
    framebuffer_front: &'fb mut [graphics_2d::Pixel],
    info: FrameBufferInfo,
}

#[derive(Debug, Clone, Copy)]
pub struct Color24(u8, u8, u8);

impl Display for Color24 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

impl Color24 {
    pub const fn grey(intensity: u8) -> Self {
        Self(intensity, intensity, intensity)
    }

    pub const fn from_hex(value: u32) -> Self {
        Self(
            ((value & 0xFF0000) >> 16) as u8,
            ((value & 0x00FF00) >> 8) as u8,
            (value & 0x0000FF) as u8,
        )
    }

    pub const fn new(red: u8, green: u8, blue: u8) -> Color24 {
        Color24(red, green, blue)
    }

    pub fn to_bgr(&self) -> [u8; 4] {
        [self.2, self.1, self.0, 0]
    }

    pub fn to_rgb(&self) -> [u8; 4] {
        [self.0, self.1, self.2, 0]
    }

    pub fn to_grey_avg(&self) -> [u8; 4] {
        let value = (((self.0 as u16) + (self.1 as u16) + (self.2 as u16)) / 3) as u8;
        [value, value, value, 0]
    }
}

impl<'fb> Vga<'fb> {
    pub fn put_char(&mut self, x: usize, y: usize, chr: char, col: (Pixel, Pixel)) {
        if let Some(font) = font8x8::BASIC_FONTS.get(chr) {
            self.draw_char_8(x, y, col.0, col.1, &font);
        } else {
            self.draw_char_8(x, y, col.0, col.1, &fonts::UNICODE_REPLACEMENT);
        }
    }

    pub fn draw_filled_square(&mut self, x: usize, y: usize, w: usize, h: usize, color: Pixel) {
        for y_pos in y..=y + h {
            for x_pos in x..=x + w {
                self.write_pixel(x_pos, y_pos, color);
            }
        }
    }

    pub fn screen_height(&self) -> usize {
        self.info.vertical_resolution
    }

    pub fn screen_width(&self) -> usize {
        self.info.horizontal_resolution
    }

    pub fn framebuffer_mut(&mut self) -> &mut [Pixel] {
        self.framebuffer_front
    }

    pub fn from_framebuffer(framebuffer: &'fb mut [Pixel], info: FrameBufferInfo) -> Self {
        let buffer = framebuffer;
        Self {
            framebuffer_front: buffer,
            info,
        }
    }

    pub fn clear(&mut self, color: Pixel) {
        self.framebuffer_front.fill(color);

        // for y in 0..self.screen_height() {
        //     for x in 0..self.screen_width() {
        //         self.write_pixel(x, y, color);
        //     }
        // }
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: Pixel) {
        let offset = self.pixel_offset(x, y);
        let p_start = offset;
        let p_end = p_start + self.info.bytes_per_pixel;

        if p_end >= self.framebuffer_front.len() || p_start >= self.framebuffer_front.len() {
            return;
        }
        {
            //let mut backbuffer = BACK_BUFFER.get().unwrap().lock();
            self.framebuffer_front[offset] = color;

            let _ = unsafe { core::ptr::read_volatile(&self.framebuffer_front[offset]) };
        }
    }

    pub fn read_pixel(&self, x: usize, y: usize) -> Pixel {
        let offset = self.pixel_offset(x, y);
        return self.framebuffer_front[offset];
    }

    pub fn byte_len(&self) -> usize {
        self.framebuffer_front.len()
    }

    pub fn shift_up(&mut self, amount: usize) {
        // let start = amount * 8 * self.row_stride * 4;
        // let end = self.byte_len() - (start);

        // for index in start..end {
        //     let byte = self.framebuffer_front[index];
        //     self.framebuffer_front[index - start] = byte;
        // }

        for y in amount..self.screen_height() {
            for x in 0..self.screen_width() {
                let pixel = self.read_pixel(x, y);
                self.write_pixel(x, y - amount, pixel);
            }
        }
    }

    fn pixel_offset(&self, x: usize, y: usize) -> usize {
        x + y * self.screen_width()
    }

    pub fn line(&mut self, color: Pixel, a: (isize, isize), b: (isize, isize)) {
        let dy = b.1 - a.1;
        let dx = b.0 - a.0;

        if dy.abs() < dx.abs() {
            if a.0 > b.0 {
                self.line_low(color, b, a);
            } else {
                self.line_low(color, a, b);
            }
        } else {
            if a.1 > b.1 {
                self.line_high(color, b, a);
            } else {
                self.line_high(color, a, b);
            }
        }
    }

    pub fn line_low(&mut self, color: Pixel, a: (isize, isize), b: (isize, isize)) {
        let mut dy = a.1 - b.1;
        let dx = a.0 - a.0;

        let mut yi = 0;

        if dy < 0 {
            yi = -1;
            dy = -dy;
        }

        let mut d = (2 * dy) - dx;
        let mut y = a.1;

        for x in a.0..b.0 {
            self.write_pixel(x.abs() as usize, y.abs() as usize, color);

            if d > 0 {
                y += yi;
                d += 2 * (dx - dy)
            } else {
                d += 2 * dy;
            }
        }
    }

    pub fn line_high(&mut self, color: Pixel, a: (isize, isize), b: (isize, isize)) {
        let dy = a.1 - b.1;
        let mut dx = a.0 - a.0;

        let mut xi = 0;

        if dx < 0 {
            xi = -1;
            dx = -dx;
        }

        let mut d = (2 * dy) - dx;
        let mut x = a.1;

        for y in a.1..b.1 {
            self.write_pixel(x.abs() as usize, y.abs() as usize, color);

            if d > 0 {
                x += xi;
                d += 2 * (dy - dx)
            } else {
                d += 2 * dx;
            }
        }
    }

    pub fn draw_char_8(&mut self, x: usize, y: usize, fg: Pixel, bg: Pixel, font: &[u8]) {
        for (index, byte) in font.iter().enumerate() {
            for bit in 0..8 {
                if ((1 << bit) & *byte) != 0 {
                    self.write_pixel(x + bit, y + index, fg);
                } else {
                    self.write_pixel(x + bit, y + index, bg);
                }
            }
        }
    }
}

pub fn get_vblank_counter() -> u8 {
    crate::arch::outb(0x3DA, 0x11);
    crate::arch::inb(0x3DD) & 0xf
}

pub fn sync() {
    while !vsync_bit() {}
}

pub fn vsync_bit() -> bool {
    ((crate::arch::inb(0x3DA) & 8) >> 7) == 1
}
