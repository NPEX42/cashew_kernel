use core::fmt::Write;

use ansi_parser::AnsiParser;
use ansi_parser::AnsiSequence;
use ansi_parser::Output;
use conquer_once::spin::OnceCell;

use crate::fonts::*;
use crate::graphics_2d::Frame;
use crate::graphics_2d::Pixel;
use crate::locked::Locked;
use crate::vga;
use crate::vga::*;

static TERMINAL: OnceCell<Locked<TerminalWriter>> = OnceCell::uninit();
static mut TERMINAL_FB: Frame = Frame::new();

const LINE_SPACING: usize = 2;

pub fn initialize() {
    TERMINAL.init_once(|| {
        Locked::new(TerminalWriter::new(
            Pixel::hex(0xCCCCCC),
            Pixel::hex(0x0000FF),
        ))
    });
}

pub fn write_fmt(args: core::fmt::Arguments) {
    TERMINAL
        .get()
        .unwrap()
        .lock()
        .write_fmt(args)
        .expect("Failed To Write To Terminal");
}

pub fn home() {
    set_y(0);
    set_x(0);
}

pub fn clear() {
    unsafe {
        TERMINAL_FB.clear(get_bg());
    }
    swap()
}

pub fn buf_height() -> usize {
    screen_height() / FONT_HEIGHT
}

pub fn buf_width() -> usize {
    screen_width() / FONT_WIDTH
}

pub fn move_y(amount: isize) {
    let mut y = y() as isize;
    if (y) < screen_height() as isize - amount {
        y += amount;
    }
    set_y(y as usize);
}

pub fn set_bg(color: Pixel) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        TERMINAL.get().unwrap().lock().bg_color = color;
    });
}

pub fn set_fg(color: Pixel) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        TERMINAL.get().unwrap().lock().fg_color = color;
    })
}

pub fn get_fg() -> Pixel {
    x86_64::instructions::interrupts::without_interrupts(|| TERMINAL.get().unwrap().lock().fg_color)
}

pub fn get_bg() -> Pixel {
    x86_64::instructions::interrupts::without_interrupts(|| TERMINAL.get().unwrap().lock().bg_color)
}

pub fn set_x(x: usize) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        TERMINAL.get().unwrap().lock().x = x;
    });
}

pub fn set_y(y: usize) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let y = y.clamp(0, screen_height() - FONT_HEIGHT);
        TERMINAL.get().unwrap().lock().y = y;
    });
}

pub fn x() -> usize {
    x86_64::instructions::interrupts::without_interrupts(|| TERMINAL.get().unwrap().lock().x)
}

pub fn y() -> usize {
    x86_64::instructions::interrupts::without_interrupts(|| TERMINAL.get().unwrap().lock().y)
}

pub fn print_custom(bitmap: &[u8]) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        TERMINAL.get().unwrap().lock().draw_bitmap(bitmap);
    });
}

pub fn set_print_newline(state: bool) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        TERMINAL.get().unwrap().lock().set_print_newline(state);
    });
}

pub fn swap() {
    unsafe {
        TERMINAL_FB.swap();
    }
}

#[macro_export]
macro_rules! print {
    ($fmt:expr, $($args:tt)*) => {
        $crate::terminal::write_fmt(format_args!($fmt, $($args)*));
        $crate::terminal::swap();
    };

    ($fmt:expr) => {
        $crate::terminal::write_fmt(format_args!(concat!($fmt)));
        $crate::terminal::swap();
    };
}

#[macro_export]
macro_rules! println {
    ($fmt:expr, $($args:tt)*) => {
        $crate::terminal::write_fmt(format_args!(concat!($fmt, "\n"), $($args)*));
        $crate::terminal::swap();
    };

    ($fmt:expr) => {
        $crate::terminal::write_fmt(format_args!(concat!($fmt, "\n")));
        $crate::terminal::swap();
    };

    () => {
        $crate::terminal::write_fmt(format_args!("\n"));
        $crate::terminal::swap();
    };
}

pub struct TerminalWriter {
    x: usize,
    y: usize,

    fg_color: Pixel,
    bg_color: Pixel,

    print_control: bool,
}

impl TerminalWriter {
    pub const fn new(fg: Pixel, bg: Pixel) -> Self {
        Self {
            bg_color: bg,
            fg_color: fg,

            x: 0,
            y: 0,

            print_control: false,
        }
    }

    pub fn put_str(&mut self, text: &str) {
        for chr in text.chars() {
            if chr == '\n' {
                if self.print_control {
                    self.put_char(chr)
                } else {
                    self.newline();
                }
            } else {
                self.put_char(chr)
            }
        }
    }

    pub fn put_char(&mut self, chr: char) {
        unsafe { &mut TERMINAL_FB }.draw_char(self.x, self.y, chr, self.fg_color, self.bg_color);
        self.x += FONT_WIDTH;
        if self.x >= vga::screen_width() {
            self.newline();
        }
    }

    pub fn draw_bitmap(&mut self, font: &[u8]) {
        vga::draw_bitmap(self.x, self.y, self.fg_color, self.bg_color, font);
        self.x += FONT_WIDTH;
        if self.x >= vga::screen_width() {
            self.newline();
        }
    }

    fn newline(&mut self) {
        self.x = 0;
        self.y += FONT_HEIGHT + LINE_SPACING;
        if self.y >= vga::screen_height() {
            self.y = vga::screen_height() - FONT_HEIGHT;
            unsafe { &mut TERMINAL_FB }.shift_up(FONT_HEIGHT);
        }
    }

    pub fn set_bg(&mut self, color: Pixel) {
        self.bg_color = color;
    }

    pub fn set_fg(&mut self, color: Pixel) {
        self.fg_color = color;
    }

    pub fn set_print_newline(&mut self, state: bool) {
        self.print_control = state;
    }

    pub fn perform_ansi(&mut self, seq: AnsiSequence) {
        match seq {
            AnsiSequence::EraseDisplay => clear(),
            AnsiSequence::CursorPos(x, y) => {
                set_x((x as usize - 1) * FONT_WIDTH);
                set_y((y as usize - 1) * FONT_HEIGHT)
            }
            _ => {}
        }
    }
}

impl Write for TerminalWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for code in s.ansi_parse() {
            match code {
                Output::TextBlock(text) => self.put_str(text),
                Output::Escape(seq) => {
                    self.perform_ansi(seq);
                }
            }
        }

        Ok(())
    }
}
