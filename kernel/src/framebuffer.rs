use alloc::vec;
use alloc::vec::Vec;

use lazy_static::lazy_static;
use spin::Mutex;
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use embedded_graphics::{
    Drawable, draw_target::DrawTarget, geometry::{self, Point}, mono_font, pixelcolor::{Rgb888, RgbColor}, prelude::Pixel, text::Text,
};

pub struct Display {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
}

impl Display {
    pub fn new(framebuffer: &'static mut FrameBuffer) -> Display {
        Self {
            info: framebuffer.info(),
            framebuffer: framebuffer.buffer_mut(),
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.info.width, self.info.height)
    }

    fn draw_pixel(&mut self, coordinates: Point, color: Rgb888) {
        // ignore any pixels that are out of bounds
        let position = match (coordinates.x.try_into(), coordinates.y.try_into()) {
            (Ok(x), Ok(y)) if x < self.info.width && y < self.info.height => Position { x, y },
            _ => return,
        };
        let color = Color {
            red: color.r(),
            green: color.g(),
            blue: color.b(),
        };
        set_pixel_in(self.framebuffer, self.info, position, color);
    }

    fn fill_region(region: &mut [u8], pixel: u32) {
        // cast framebuffer to u32 slice and fill
        let (prefix, aligned, suffix) = unsafe { region.align_to_mut::<u32>() };
        aligned.fill(pixel);
        // handle unaligned edges
        for byte in prefix.iter_mut().chain(suffix.iter_mut()) {
            *byte = 0;
        }
    }

    pub fn clear(&mut self, color: Color) {
        // pack color into a u32 pixel value
        let pixel = match self.info.pixel_format {
            PixelFormat::Bgr => u32::from_le_bytes([color.blue, color.green, color.red, 0]),
            PixelFormat::Rgb => u32::from_le_bytes([color.red, color.green, color.blue, 0]),
            _ => 0,
        };
        Display::fill_region(self.framebuffer, pixel);
    }

    pub fn scroll_up(&mut self, rows: usize, bg: Color) {
        let row_bytes = self.info.stride * self.info.bytes_per_pixel * rows;

        // shift everything up by 'rows' pixels
        self.framebuffer.copy_within(row_bytes.., 0);

        // clear the bottom 'rows' pixels
        let bottom_start = self.framebuffer.len() - row_bytes;
        let pixel = match self.info.pixel_format {
            PixelFormat::Bgr => u32::from_le_bytes([bg.blue, bg.green, bg.red, 0]),
            PixelFormat::Rgb => u32::from_le_bytes([bg.red, bg.green, bg.blue, 0]),
            _ => 0,
        };
        Display::fill_region(&mut self.framebuffer[bottom_start..], pixel);
    }

    pub fn split_at_line(self, line_index: usize) -> (Self, Self) {
        assert!(line_index < self.info.height);

        let byte_offset = line_index * self.info.stride * self.info.bytes_per_pixel;
        let (first_buffer, second_buffer) = self.framebuffer.split_at_mut(byte_offset);

        let first = Self {
            framebuffer: first_buffer,
            info: FrameBufferInfo {
                byte_len: byte_offset,
                height: line_index,
                ..self.info
            },
        };
        let second = Self {
            framebuffer: second_buffer,
            info: FrameBufferInfo {
                byte_len: self.info.byte_len - byte_offset,
                height: self.info.height - line_index,
                ..self.info
            },
        };

        (first, second)
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;

    /// Drawing operations can never fail.
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>
    {
        for Pixel(coordinates, color) in pixels.into_iter() {
            self.draw_pixel(coordinates, color);
        }
        Ok(())
    }
}

impl geometry::OriginDimensions for Display {
    fn size(&self) -> geometry::Size {
        geometry::Size::new(
            self.info.width.try_into().unwrap(),
            self.info.height.try_into().unwrap(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub fn set_pixel_in(
    framebuffer: &mut [u8],
    info: FrameBufferInfo,
    position: Position,
    color: Color,
) {
    // calculate offset to the first byte of pixel
    let byte_offset = {
        // use stride to calculate pixel offset of target line
        let line_offset = position.y * info.stride;
        // add x position to get the absolute pixel offset in buffer
        let pixel_offset = line_offset + position.x;
        // convert to byte offset
        pixel_offset * info.bytes_per_pixel
    };

    // set pixel based on color format
    let pixel_bytes = &mut framebuffer[byte_offset..];
    match info.pixel_format {
        PixelFormat::Rgb => {
            pixel_bytes[0] = color.red;
            pixel_bytes[1] = color.green;
            pixel_bytes[2] = color.blue;
        }
        PixelFormat::Bgr => {
            pixel_bytes[0] = color.blue;
            pixel_bytes[1] = color.green;
            pixel_bytes[2] = color.red;
        }
        PixelFormat::U8 => {
            // use a simple average-based grayscale transform
            let gray = color.red / 3 + color.green / 3 + color.blue / 3;
            pixel_bytes[0] = gray;
        }
        other => panic!("unknown pixel format {:?}", other),
    }
}

// interface for TTY terminals //

// public mutexes for sharing across different terminals
lazy_static! {
    pub static ref DISPLAY: Mutex<Option<Display>> = Mutex::new(None);
    pub static ref CONSOLE: Mutex<Option<TextConsole>> = Mutex::new(None);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextCell {
    pub character: char,
    pub fg: Rgb888,
    pub bg: Rgb888,
}

impl TextCell {
    pub fn empty(bg: Rgb888) -> Self {
        Self {
            character: ' ',
            fg: Rgb888::WHITE,
            bg,
        }
    }
}

pub struct TextConsole {
    col: usize,
    row: usize,
    cols: usize,
    rows: usize,
    fg: Rgb888,
    bg: Rgb888,
    buffer: Vec<TextCell>,
    dirty: Vec<bool>,
}

impl TextConsole {
    pub fn new(width: usize, height: usize, fg: Rgb888, bg: Rgb888) -> Self {
        // TODO: add support for changing fonts
        let cols = width / 10;
        let rows = height / 20;

        let buffer = vec![TextCell::empty(bg); cols * rows];
        let dirty = vec![false; buffer.len()];

        Self { col: 0, row: 0, cols, rows, fg, bg, buffer, dirty }
    }

    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.col = 0,
            _ => {
                // end of line, wrap to new line
                if self.col >= self.cols {
                    self.newline();
                }

                let index = self.row * self.cols + self.col;
                self.buffer[index] = TextCell {
                    character: c,
                    fg: self.fg,
                    bg: self.bg,
                };
                self.dirty[index] = true;

                self.col += 1;
            }
        }
    }

    fn newline(&mut self) {
        self.col = 0;
        if self.row < self.rows - 1 {
            self.row += 1;
        } else {
            self.scroll();
        }
    }

    fn scroll(&mut self) {
        DISPLAY.lock().as_mut().unwrap().scroll_up(20, Color { red: 0, green: 0, blue: 0 });

        self.dirty.fill(true);
    }

    pub fn clear(&mut self) {
        DISPLAY.lock().as_mut().unwrap().clear(Color { red: 0, green: 0, blue: 0 });
        self.col = 0;
        self.row = 0;

        self.dirty.fill(true);
    }

    pub fn flush(&mut self) {
        if let Some(ref mut display) = *DISPLAY.lock() {
            for row in 0..self.rows {
                for col in 0..self.cols {
                    let index = row * self.cols + col;
                    if !self.dirty[index] { continue; }

                    let cell = self.buffer[index];
                    let x = (col * 10) as i32;
                    let y = (row * 20) as i32;

                    let style = mono_font::MonoTextStyleBuilder::new()
                        .font(&mono_font::ascii::FONT_10X20)
                        .text_color(cell.fg)
                        .background_color(cell.bg)
                        .build();

                    let mut buf = [0; 4];
                    let text_str = cell.character.encode_utf8(&mut buf);
                    let _ = Text::new(text_str, Point::new(x, y + 15), style).draw(display);

                    self.dirty[index] = false;
                }
            }
        }
    }
}

impl core::fmt::Write for TextConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

// print macros //

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    if let Some(ref mut console) = *CONSOLE.lock() {
        console.write_fmt(args).unwrap();
        console.flush();
    }
}

#[doc(hidden)]
pub fn _clear() {
    if let Some(ref mut console) = *CONSOLE.lock() {
        console.clear();
        console.flush();
    }
}

/// Prints to the screen through the tty interface.
#[macro_export]
macro_rules! tty_print {
    ($($arg:tt)*) => {
        $crate::framebuffer::_print(format_args!($($arg)*))
    };
}

/// Prints to the screen through the tty interface, with a newline.
#[macro_export]
macro_rules! tty_println {
    () => ($crate::tty_print!("\n"));
    ($fmt:expr) => ($crate::tty_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::tty_print!(
        concat!($fmt, "\n"), $($arg)*));
}

/// Clears the framebuffer (and therefore the screen).
#[macro_export]
macro_rules! tty_clear {
    () => ($crate::framebuffer::_clear());
}
