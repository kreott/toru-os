#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader_api::BootloaderConfig;
use bootloader_api::config::Mapping;

extern crate alloc;

pub mod framebuffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod tty;
pub mod macros;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};


pub fn main_inits() {
    serial_println!("Running initializations...");

    serial_println!("initializing Global Descriptor Table...");
    gdt::init();

    serial_println!("initializing Interrupt Descriptor Table...");
    interrupts::init_idt();

    serial_println!("initializing PICs...");
    unsafe { 
        interrupts::PICS.lock().initialize();
        // unmask IRQ0 (timer) and IRQ1 (keyboard), for UEFI
        interrupts::PICS.lock().write_masks(0xFC, 0xFF);
    };

    serial_println!("enabling interrupts...");
    x86_64::instructions::interrupts::enable();

    serial_println!("Initializations complete!");
}

// TESTING STUFF
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
bootloader_api::entry_point!(test_kernel_main, config = &BOOTLOADER_CONFIG);

#[cfg(test)]
use bootloader_api::BootInfo;

/// Entry point for 'cargo test'
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    main_inits();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}