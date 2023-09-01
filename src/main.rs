fn main() {
    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    // choose whether to start the UEFI or BIOS image
    let uefi = false;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
    } else {
        cmd.arg("-drive")
            .arg(format!("format=raw,file={bios_path}"));
    }

    // Debugging
    cmd.arg("-s");
    cmd.arg("-S");

    // Run using Windows Hypervisor Platform (speeds up execution)
    // See Step 3: https://learn.microsoft.com/en-us/windows/wsl/install-manual#step-3---enable-virtual-machine-feature
    //cmd.arg("-accel");
    //cmd.arg("whpx");

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
