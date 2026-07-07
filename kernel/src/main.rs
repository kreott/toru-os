#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

extern crate alloc;

use alloc::vec::Vec;
use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping};
use embedded_graphics::{
    geometry::Point, pixelcolor::{Rgb888, RgbColor, WebColors},
};
use x86_64::VirtAddr;

use kernel::allocator;
use kernel::memory;
use kernel::framebuffer;
use kernel::memory::BootInfoFrameAllocator;
use kernel::macros::*;

bootloader_api::entry_point!(kernel_main, config = &kernel::BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    
    kernel::main_inits();

    // initialize memory and heap
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { 
        BootInfoFrameAllocator::init(&boot_info.memory_regions) 
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    let x = alloc::boxed::Box::new(41);
    serial_println!("heap_value as {:?}", x);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    serial_println!("vec at {:p}", vec.as_slice());

    #[cfg(test)]
    test_main();

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let mut display = framebuffer::Display::new(framebuffer);
        let mut painter = framebuffer::Painter::new(&mut display);

        painter.clear(Rgb888::BLACK);
        painter.circle(Point::new(100, 50), 300, Rgb888::CSS_LIGHT_PINK);
        painter.text("Hej", Point::new(200, 200), Rgb888::BLACK);
    }

    kernel::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    kernel::hlt_loop();
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