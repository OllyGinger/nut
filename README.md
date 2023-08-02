# 🥜 Nut
This is an attempt to boot and bring up a basic kernel in Rust. This is more than likely going to fail before I can get any progress, but hopefully I'll learn something on the way.

## Goals
- [ ] Create a BIOS bootloader that can launch my yet to be made kernel on an x86-64 CPU
- [ ] Create a kernel that can print something to the screen

## Running
* Install [`QEmu`](https://www.minitool.com/partition-disk/qemu-for-windows.html) - Make sure the add the install path to `%PATH%`
* Install [`rustup`](https://www.rust-lang.org/tools/install)
* Install the nightly build of Rust: `rustup toolchain install nightly`
* Switch to the directory that contains this code and run `cargo run`. This will perform the following;
    * Build bootloader\bios\stage0 - MBR loader
    * Build bootloader\bios\stage1 - Second stage loader
    * Package into a raw MBR image
    * Launch the Nut kernel via the bootloader in QEmu

## Bootloader
The bootloader is mutli-stage. The stages are;
| Stage | Mode | Description |
| -- | -- | -- |
| Stage 0 | Real Mode (16 bit) | This stage has to fit inside the MBR (Master boot record), so it can only be 512 bytes long. This stage performs the following; <ul><li>Enable the A20 line</li><li>Find the next stage and call it</li></ul> |
| Stage 1 | Real Mode (16 bit) | This stage can be up to 1 magabyte in size, this is because we still only have 16 bits of address space, but Real Mode uses `segment:offset` addressing. This allows us to use up to 1 megabyte of memory in total using a combination of the following segment registers and the offset: <ul><li>CS (Code Segment) - Store base segment address for code</li><li>DS (Data Segment) - Store base segment address for data</li><li>ES (Extra Segment) - Store base segment address for anything</li><li>SS (Stack Segment) - Store base segment address for the stack</li></ul>The main purpose of this stage is to parse the FAT32 partition to be able to locate the rest of the bootloader which are stored as files in the boot image.
| Stage 2 | Protected Mode (32 bit) | This stage can access up to 4 Gigabytes of address space, but we won't need anywhere near that much. This stage is only really used to switch to Long Mode (64 bit) and locate and call the next and final stage. |
| Stage 3 | Long Mode (64 bit) | This is the final stage, and it's main task is to load any configuration file we might have to set up the Kernel, and to actually jump into the Kernel.

## Useful links
### General
* https://wiki.osdev.org/Expanded_Main_Page
* https://os.phil-opp.com

### Bootloader
* https://github.com/rust-osdev/bootloader/tree/main
* https://neosmart.net/wiki/mbr-boot-process/
* http://www.brokenthorn.com/Resources/OSDevIndex.html

