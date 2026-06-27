//! Runs both the UEFI and BIOS images and their appropriate tests.

use std::{env, path::PathBuf, process::{self, Command}};
use disk_builder::DiskImageBuilder;

fn run_qemu(image: &PathBuf, uefi: bool) -> i32 {
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive");
    qemu.arg(format!("format=raw,file={}", image.display()));
    if uefi {
        qemu.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    }
    qemu.arg("-serial").arg("stdio");
    qemu.arg("-display").arg("none");
    qemu.arg("-device").arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

    qemu.status().unwrap().code().unwrap_or(-1)
}

fn main() {
    let test_binary = PathBuf::from(env::args().nth(1).expect("no test binary path given"));

    let uefi_image = test_binary.with_extension("uefi.img");
    let bios_image = test_binary.with_extension("bios.img");

    DiskImageBuilder::new(test_binary.clone())
        .create_uefi_image(&uefi_image)
        .unwrap();

    DiskImageBuilder::new(test_binary)
        .create_bios_image(&bios_image)
        .unwrap();

    let uefi_exit = run_qemu(&uefi_image, true);
    if uefi_exit != 33 {
        println!("UEFI test failed");
        process::exit(1);
    }
    let bios_exit = run_qemu(&bios_image, false);
    if bios_exit != 33 {
        println!("BIOS test failed");
        process::exit(1);
    }

    process::exit(match (uefi_exit, bios_exit) {
        (33, 33) => 0,
        _ => 1,
    });
}