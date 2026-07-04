use disk_builder::DiskImageBuilder;
use std::{env, path::PathBuf};

fn main() {
    let kernel_path = PathBuf::from("target/x86_64-unknown-none/debug/kernel");
    let disk_builder = DiskImageBuilder::new(kernel_path);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let uefi_path = out_dir.join("toru_os-uefi.img");
    let bios_path = out_dir.join("toru_os-bios.img");

    disk_builder.create_uefi_image(&uefi_path).unwrap();
    disk_builder.create_bios_image(&bios_path).unwrap();

    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
    println!("cargo:rerun-if-changed=kernel/src");
}