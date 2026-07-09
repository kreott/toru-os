use embedded_graphics::pixelcolor::{Rgb888, RgbColor};

use crate::framebuffer::{DISPLAY, TextConsole};

use crate::macros::*;

pub async fn tty_task() {
    tty_clear!();
    tty_println!("Hello world!");
}