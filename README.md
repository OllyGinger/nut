# 🥜 Nut
This is an attempt to boot and bring up a basic kernel in Rust. This is more than likely going to fail before I can get any progress, but hopefully I'll learn something on the way.

## Goals
- [ ] Create a BIOS bootloader that can launch my yet to be made kernel on an x86-64 CPU
- [ ] Create a kernel that can print something to the screen

## Running
* Install QEmu - https://www.minitool.com/partition-disk/qemu-for-windows.html
* Run `cargo run`. This will perform the following;
** Build bootloader\bios\stage0 - MBR loader
** Build bootloader\bios\stage1 - Second stage loader
** Package into a raw MBR image
** Launch QEmu

## Useful links
### General
* https://wiki.osdev.org/Expanded_Main_Page
* https://os.phil-opp.com

## Bootloader
* https://github.com/rust-osdev/bootloader/tree/main
* https://neosmart.net/wiki/mbr-boot-process/
* http://www.brokenthorn.com/Resources/OSDevIndex.html

