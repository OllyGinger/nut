# 🥜 Nut
This is an attempt to boot and bring up a basic kernel in Rust. This is more than likely going to fail before I can get any progress, but hopefully I'll learn something on the way.

# Goals
- [ ] Create a BIOS bootloader that can launch my yet to be made kernel on an x86-64 CPU
- [ ] Create a kernel that can print something to the screen

# Running
* Install QEmu - https://www.minitool.com/partition-disk/qemu-for-windows.html
* Run `cargo build`. This will perform the following;
** Build bootloader\bios\stage0 - Stage 0 boot-sector

# Notes
Current main binary is the first stage of the boot loader, and needs to fit in 512 byte boot sector
* Build using cargo: `cargo build --release -Zbuild-std=core --target .\i386-code16-boot-sector.json -Zbuild-std-features=compiler-builtins-mem`
* Then run: `"C:\Program Files\LLVM\bin\llvm-objcopy.exe" -I elf32-i386 -O binary target\i386-code16-boot-sector\release\nut target\disk_image.bin`
* Then to run in qemu: `qemu-system-x86_64 -drive format=raw,file=target\disk_image.bin`

# Useful links
* https://wiki.osdev.org/Expanded_Main_Page
* https://os.phil-opp.com
* https://github.com/rust-osdev/bootloader/tree/main

