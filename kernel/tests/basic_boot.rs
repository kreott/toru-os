#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader_api::BootInfo;
use kernel::macros::*;
use kernel::{QemuExitCode, exit_qemu};

bootloader_api::entry_point!(kernel_test_main);

fn kernel_test_main(_boot_info: &'static mut BootInfo) -> ! {
    test_main();

    loop {}
}

#[test_case]
fn test_serial_print() {
    serial_print!("serial_print test")
}

#[test_case]
fn test_serial_println() {
    serial_println!("serial_println test")
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info.message());
    exit_qemu(QemuExitCode::Failed);
    loop {}
}