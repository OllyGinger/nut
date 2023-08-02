use std::{collections::BTreeMap, path::Path, process::Command};

use crate::disk::FileDataSource;

mod disk;
mod fat;

pub fn main() {
    // Build the boot partition (FAT32) - the files that make up the disk:
    // * Final stages of the bootloader
    // * Kernel
    // * Misc files
    let pathbuf = Path::new("target").join("disk.img");
    let image_path = pathbuf.as_path();

    let mut internal_files = BTreeMap::new();
    internal_files.insert(
        "boot-stage-2",
        FileDataSource::File(Path::new(env!("BIOS_BOOT_STAGE2_PATH")).to_path_buf()),
    );

    // Create the FAT32 partition
    let fat_partition = disk::create_fat_filesystem_image(internal_files).unwrap();

    disk::create_mbr_disk(
        Path::new(env!("BIOS_BOOT_STAGE0_PATH")),
        Path::new(env!("BIOS_BOOT_STAGE1_PATH")),
        fat_partition.path(),
        image_path,
    );

    // Start QEmu
    println!("Starting QEmu. Image: {}", image_path.display());
    Command::new("qemu-system-x86_64")
        .arg("-drive")
        .arg(format!("format=raw,file={}", image_path.display()))
        .output()
        .expect("Failed to start qemu-system-x86_64. Is it installed in the PATH?");
}
