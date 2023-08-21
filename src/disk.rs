use crate::fat;
use mbrman::BOOT_ACTIVE;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Seek,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;
const SECTOR_SIZE: u32 = 512;

#[derive(Clone)]
/// Defines a data source, either a source `std::path::PathBuf`, or a vector of bytes.
pub enum FileDataSource {
    File(PathBuf),
    Data(Vec<u8>),
}

impl FileDataSource {
    /// Get the length of the inner data source
    pub fn len(&self) -> Result<u64, String> {
        Ok(match self {
            FileDataSource::File(path) => fs::metadata(path).unwrap().len(),
            FileDataSource::Data(v) => v.len() as u64,
        })
    }

    pub fn copy_to(&self, target: &mut dyn std::io::Write) -> Result<(), String> {
        match self {
            FileDataSource::File(file_path) => {
                std::io::copy(&mut fs::File::open(file_path).unwrap(), target).unwrap();
            }
            FileDataSource::Data(contents) => {
                let mut cursor = std::io::Cursor::new(contents);
                std::io::copy(&mut cursor, target).unwrap();
            }
        };

        Ok(())
    }
}

pub fn create_mbr_disk(
    stage0_path: &Path,
    stage1_path: &Path,
    boot_partition_path: &Path,
    image_output_path: &Path,
) {
    let mut boot_sector = File::open(stage0_path).unwrap();
    let mut mbr = mbrman::MBR::read_from(&mut boot_sector, SECTOR_SIZE).unwrap();

    let mut stage1_file = File::open(stage1_path).unwrap();
    let stage1_len = stage1_path.metadata().unwrap().len();
    let stage1_start_sector = 1;
    let stage1_num_sectors = ((stage1_len - 1) / u64::from(SECTOR_SIZE) + 1)
        .try_into()
        .unwrap();
    mbr[1] = mbrman::MBRPartitionEntry {
        boot: BOOT_ACTIVE,
        starting_lba: stage1_start_sector,
        sectors: stage1_num_sectors,
        sys: 0x20,
        first_chs: mbrman::CHS::empty(),
        last_chs: mbrman::CHS::empty(),
    };

    // Add a new partition for extra files
    let mut boot_partition = File::open(boot_partition_path).unwrap();
    let boot_partition_start_sector = stage1_start_sector + stage1_num_sectors;
    let boot_partition_size = boot_partition.metadata().unwrap().len();
    mbr[2] = mbrman::MBRPartitionEntry {
        boot: BOOT_ACTIVE,
        starting_lba: boot_partition_start_sector,
        sectors: ((boot_partition_size - 1) / u64::from(SECTOR_SIZE) + 1)
            .try_into()
            .unwrap(),
        sys: 0x0c, // FAT32 with LBA
        first_chs: mbrman::CHS::empty(),
        last_chs: mbrman::CHS::empty(),
    };

    let mut disk = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(image_output_path)
        .unwrap();

    // Write the image file
    mbr.write_into(&mut disk).unwrap();
    std::io::copy(&mut stage1_file, &mut disk).unwrap();

    // Write the fat32 partition
    disk.seek(std::io::SeekFrom::Start(
        (boot_partition_start_sector * SECTOR_SIZE).into(),
    ))
    .unwrap();
    std::io::copy(&mut boot_partition, &mut disk).unwrap();
}

pub fn create_fat_filesystem_image(
    internal_files: BTreeMap<&str, FileDataSource>,
) -> Result<NamedTempFile, String> {
    let out_file = NamedTempFile::new().unwrap();
    let mut local_files: BTreeMap<&str, _> = BTreeMap::new();

    for k in &internal_files {
        if local_files.insert(k.0, k.1).is_some() {
            return Err(format!("Attempted to overwrite file: {}", k.0));
        }
    }

    fat::create_fat_filesystem(local_files, out_file.path()).unwrap();
    println!("FAT FILE: {}", out_file.path().display());
    Ok(out_file)
}
