use std::process::Command;

pub fn main() {
    // qemu-system-x86_64 -drive format=raw,file=%NUT_BOOTLOADER_IMAGE%
    let path = env!("NUT_BOOTLOADER_IMAGE");
    println!("Starting QEmu. Image: {path}");

    Command::new("qemu-system-x86_64")
        .arg("-drive")
        .arg(format!("format=raw,file={path}"))
        .output()
        .expect("Failed to start qemu-system-x86_64. Is it installed in the PATH?");
}
