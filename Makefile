.PHONY: uefi bios test

uefi:
	cargo build -p kernel --target x86_64-unknown-none -Zbuild-std=core,compiler_builtins,alloc -Zbuild-std-features=compiler-builtins-mem
	cargo run --bin qemu-uefi

bios:
	cargo build -p kernel --target x86_64-unknown-none -Zbuild-std=core,compiler_builtins,alloc -Zbuild-std-features=compiler-builtins-mem
	cargo run --bin qemu-bios

test:
	cargo build -p kernel --target x86_64-unknown-none -Zbuild-std=core,compiler_builtins,alloc -Zbuild-std-features=compiler-builtins-mem
	cargo test -p kernel --target x86_64-unknown-none