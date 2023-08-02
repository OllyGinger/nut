use crate::disk::FileDataSource;
use std::collections::BTreeMap;
use std::fs;

pub fn create_fat_filesystem(
    files: BTreeMap<&str, &FileDataSource>,
    out_file: &std::path::Path,
) -> Result<(), String> {
    const MB: u64 = 1024 * 1024;

    let mut size_needed = 0;
    for source in files.values() {
        size_needed += source.len()?;
    }

    // Create a new filesystem image
    let fat_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_file)
        .unwrap();
    let fat_size_padded_rounded = ((size_needed + 1024 * 64 - 1) / MB + 1) * MB;
    fat_file.set_len(fat_size_padded_rounded).unwrap();

    let label = *b"NUT_OS_BOOT";
    let format_options = fatfs::FormatVolumeOptions::new().volume_label(label);
    fatfs::format_volume(&fat_file, format_options).unwrap();
    let filesystem = fatfs::FileSystem::new(&fat_file, fatfs::FsOptions::new()).unwrap();
    let root_dir = filesystem.root_dir();

    // Copy the files to the filesystem
    add_files_to_image(&root_dir, files)
}

pub fn add_files_to_image(
    root_dir: &fatfs::Dir<&std::fs::File>,
    files: BTreeMap<&str, &FileDataSource>,
) -> Result<(), String> {
    for (target_path_raw, source) in files {
        let target_path = std::path::Path::new(target_path_raw);
        // create parent directories
        let ancestors: Vec<_> = target_path.ancestors().skip(1).collect();
        for ancestor in ancestors.into_iter().rev().skip(1) {
            root_dir
                .create_dir(&ancestor.display().to_string())
                .unwrap();
        }

        let mut new_file = root_dir.create_file(target_path_raw).unwrap();
        new_file.truncate().unwrap();

        source.copy_to(&mut new_file).unwrap();
    }

    Ok(())
}
