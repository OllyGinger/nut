// Based on https://github.com/rust-osdev/bootloader/blob/main/bios/stage-2/src/main.rs

use core::fmt::Write;

use crate::{fat, protected_mode};
use bootloader_x86_64_bios::{
    disk_access::{self, Read, Seek},
    print,
};

fn try_load_file(
    file_name: &str,
    dst: *mut u8,
    fs: &mut fat::FileSystem<disk_access::DiskAccess>,
    disk: &mut disk_access::DiskAccess,
    disk_buffer: &mut disk_access::AlignedArrayBuffer<16384>,
) -> Option<u64> {
    let disk_buffer_size = disk_buffer.buffer.len();

    let file = fs.find_file_in_root_dir(file_name, disk_buffer)?;

    let file_size = file.file_size().into();
    //writeln!(print::Writer, "File {:?}", file).unwrap();

    let mut total_offset = 0;
    for cluster in fs.file_clusters(&file) {
        writeln!(print::Writer, "xxxxx: ").unwrap();

        let cluster = cluster.unwrap();
        let cluster_start = cluster.start_offset;
        let cluster_end = cluster_start + u64::from(cluster.len_bytes);
        writeln!(print::Writer, "Cluster: {cluster_start}").unwrap();

        let mut offset = 0;
        loop {
            let range_start = cluster_start + offset;
            //writeln!(print::Writer, "range_start: {range_start}").unwrap();
            if range_start >= cluster_end {
                //writeln!(print::Writer, "range break;").unwrap();
                break;
            }
            let range_end = u64::min(
                range_start + u64::try_from(disk_buffer_size).unwrap(),
                cluster_end,
            );
            //writeln!(print::Writer, "range_end: {range_start}").unwrap();
            let len = range_end - range_start;
            //writeln!(print::Writer, "len: {range_start}").unwrap();

            disk.seek(disk_access::SeekFrom::Start(range_start));
            disk.read_exact_into(disk_buffer_size, disk_buffer);
            let slice = &disk_buffer.buffer[..usize::try_from(len).unwrap()];
            unsafe {
                protected_mode::copy_to_protected_mode(dst.wrapping_add(total_offset), slice)
            };
            let written =
                unsafe { protected_mode::read_from_protected_mode(dst.wrapping_add(total_offset)) };
            assert_eq!(slice[0], written);

            offset += len;
            total_offset += usize::try_from(len).unwrap();
        }
    }

    writeln!(print::Writer, "End of file...").unwrap();

    Some(file_size)
}

pub fn load_file(
    file_name: &str,
    dst: *mut u8,
    fs: &mut fat::FileSystem<disk_access::DiskAccess>,
    disk: &mut disk_access::DiskAccess,
    disk_buffer: &mut disk_access::AlignedArrayBuffer<16384>,
) -> u64 {
    try_load_file(file_name, dst, fs, disk, disk_buffer).expect("file not found")
}