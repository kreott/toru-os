#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader_api::BootInfo;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use x86_64::VirtAddr;

use kernel::{allocator, framebuffer::TextConsole, task::{Task, executor::Executor, keyboard, tty}};
use kernel::memory;
use kernel::framebuffer;
use kernel::memory::BootInfoFrameAllocator;
use kernel::macros::*;

bootloader_api::entry_point!(kernel_main, config = &kernel::BOOTLOADER_CONFIG);

#[allow(unreachable_code)] // allow unreachable code to avoid warnings after 'executor.run()' as it never returns
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    
    kernel::main_inits();

    // initialize memory and heap
    serial_println!("initializing memory and heap...");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { 
        BootInfoFrameAllocator::init(&boot_info.memory_regions) 
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // initialize global display for drawing
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let display = framebuffer::Display::new(framebuffer);
        let (width, height) = display.dimensions();
        *framebuffer::DISPLAY.lock() = Some(display);
        *framebuffer::CONSOLE.lock() = Some(TextConsole::new(width, height, Rgb888::WHITE, Rgb888::BLACK));
    }

    serial_println!("Initializations successful!");

    // initialize async executor
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.spawn(Task::new(tty::tty_task()));
    executor.run();

    #[cfg(test)]
    test_main();

    kernel::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    serial_println!("async number: {}", number);
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