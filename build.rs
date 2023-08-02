use async_process::Command;
use futures::executor::block_on;
use futures_concurrency::future::Join;
use std::path::{Path, PathBuf};

const BOOTLOADER_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let _ = block_on(bios_main()).join();
}

async fn bios_main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let (bios_boot_stage0_path, bios_boot_stage1_path, bios_boot_stage2_path) = (
        build_bios_boot_stage0(&out_dir),
        build_bios_boot_stage1(&out_dir),
        build_bios_boot_stage2(&out_dir),
    )
        .join()
        .await;
    println!(
        "cargo:rustc-env=BIOS_BOOT_STAGE0_PATH={}",
        bios_boot_stage0_path.display()
    );
    println!(
        "cargo:rustc-env=BIOS_BOOT_STAGE1_PATH={}",
        bios_boot_stage1_path.display()
    );
    println!(
        "cargo:rustc-env=BIOS_BOOT_STAGE2_PATH={}",
        bios_boot_stage2_path.display()
    );
    println!(
        "cargo:warning=BIOS_BOOT_STAGE2_PATH={}",
        bios_boot_stage2_path.display()
    );
}

async fn build_bios_boot_stage0(out_dir: &Path) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install").arg("bootloader-x86_64-bios-stage0");
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("bios")
        .join("stage0");
    if local_path.exists() {
        cmd.arg("--path").arg(&local_path);
        println!("cargo:rerun-if-changed={}", local_path.display());
    } else {
        cmd.arg("--version").arg(BOOTLOADER_VERSION);
    }
    cmd.arg("--locked");
    cmd.arg("--target").arg("i386-code16-stage0.json");
    cmd.arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem");
    cmd.arg("--root").arg(out_dir);
    cmd.env_remove("RUSTFLAGS");
    cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
    cmd.env_remove("RUSTC_WORKSPACE_WRAPPER");

    let status = cmd
        .status()
        .await
        .expect("failed to run cargo install for bios stage0");
    let elf_path = if status.success() {
        let path = out_dir.join("bin").join("bootloader-x86_64-bios-stage0");
        assert!(
            path.exists(),
            "bios boot stage0 executable does not exist after building"
        );
        path
    } else {
        panic!("failed to build bios boot stage0");
    };
    convert_elf_to_bin(elf_path).await
}

async fn build_bios_boot_stage1(out_dir: &Path) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install").arg("bootloader-x86_64-bios-stage1");
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("bios")
        .join("stage1");
    if local_path.exists() {
        // local build
        cmd.arg("--path").arg(&local_path);
        println!("cargo:rerun-if-changed={}", local_path.display());
        println!(
            "cargo:rerun-if-changed={}",
            local_path.with_file_name("common").display()
        );
    } else {
        cmd.arg("--version").arg(BOOTLOADER_VERSION);
    }
    cmd.arg("--locked");
    cmd.arg("--target").arg("i386-code16-stage0.json");
    cmd.arg("--profile").arg("stage1");
    cmd.arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem");
    cmd.arg("--root").arg(out_dir);
    cmd.env_remove("RUSTFLAGS");
    cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
    cmd.env_remove("RUSTC_WORKSPACE_WRAPPER"); // used by clippy
    let status = cmd
        .status()
        .await
        .expect("failed to run cargo install for bios stage1");
    let elf_path = if status.success() {
        let path = out_dir.join("bin").join("bootloader-x86_64-bios-stage1");
        assert!(
            path.exists(),
            "bios stage1 executable does not exist after building"
        );
        path
    } else {
        panic!("failed to build bios stage1");
    };
    convert_elf_to_bin(elf_path).await
}

async fn build_bios_boot_stage2(out_dir: &Path) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install").arg("bootloader-x86_64-bios-stage2");
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bootloader")
        .join("bios")
        .join("stage2");
    if local_path.exists() {
        // local build
        cmd.arg("--path").arg(&local_path);
        println!("cargo:rerun-if-changed={}", local_path.display());
    } else {
        cmd.arg("--version").arg(BOOTLOADER_VERSION);
    }
    cmd.arg("--locked");
    cmd.arg("--target").arg("i386-unknown-none.json");
    cmd.arg("--profile").arg("stage2");
    cmd.arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem");
    cmd.arg("--root").arg(out_dir);
    cmd.env_remove("RUSTFLAGS");
    cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
    cmd.env_remove("RUSTC_WORKSPACE_WRAPPER"); // used by clippy
    let status = cmd
        .status()
        .await
        .expect("failed to run cargo install for bios stage2");
    let elf_path = if status.success() {
        let path = out_dir.join("bin").join("bootloader-x86_64-bios-stage2");
        assert!(
            path.exists(),
            "bios stage2 executable does not exist after building"
        );
        path
    } else {
        panic!("failed to build bios stage2");
    };
    convert_elf_to_bin(elf_path).await
}

async fn convert_elf_to_bin(elf_path: PathBuf) -> PathBuf {
    let flat_binary_path = elf_path.with_extension("bin");

    // convert first stage to binary
    let mut cmd = Command::new("llvm-objcopy");
    cmd.arg("-I").arg("elf64-x86-64");
    cmd.arg("-O").arg("binary");
    cmd.arg("--binary-architecture=i386:x86-64");
    cmd.arg(&elf_path);
    cmd.arg(&flat_binary_path);
    let output = cmd
        .output()
        .await
        .expect("failed to execute llvm-objcopy command");
    if !output.status.success() {
        panic!(
            "objcopy failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    flat_binary_path
}
