#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]


use core::panic::PanicInfo;

use bootloader_api::BootInfo;
use embedded_graphics::{
    geometry::Point, pixelcolor::{Rgb888, RgbColor, WebColors},
};

use kernel::macros::*;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use kernel::framebuffer;

    kernel::main_inits();

    #[cfg(test)]
    test_main();

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let mut display = framebuffer::Display::new(framebuffer);
        let mut painter = framebuffer::Painter::new(&mut display);

        painter.clear(Rgb888::BLACK);
        painter.circle(Point::new(100, 50), 300, Rgb888::CSS_LIGHT_PINK);
        painter.text("Hiiiii :3", Point::new(200, 200), Rgb888::BLACK);
    }

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}