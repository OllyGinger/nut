#![no_std]
#![no_main]

use core::{fmt::Write, slice};

use crate::partition::{PartitionTableEntry, PartitionType};
use bootloader_x86_64_bios::{disk_access, fail, print, BiosFramebufferInfo, BiosInfo, Region};
use load_file::load_file;
use protected_mode::{enter_unreal_mode, protected_mode_jump_to_stage2};

mod fat;
mod load_file;
mod partition;
mod protected_mode;
mod vesa;

/// We use this partition type to store the second bootloader stage;
const BOOTLOADER_SECOND_STAGE_PARTITION_TYPE: u8 = 0x20;

// 1MB Location that stage 2 loader will be copied to
const STAGE_2_DST: *mut u8 = 0x0010_0000 as *mut u8;
const STAGE_3_DST: *mut u8 = 0x0013_0000 as *mut u8;

static mut DISK_BUFFER: disk_access::AlignedArrayBuffer<0x4000> = disk_access::AlignedArrayBuffer {
    buffer: [0; 0x4000],
};

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(disk_number: u16, partition_table_start: *const u8) -> ! {
    // Enter unreal mode so we can reference more memory (we're still in real mode though)
    enter_unreal_mode();

    print::print_str("Starting Stage 1...\n");

    // Read the partitions from the partition table
    let partitions = {
        const MAX_PARTITIONS: usize = 4;
        const PARTITION_ENTRY_SIZE: usize = 16;

        let mut entries = [PartitionTableEntry::empty(); MAX_PARTITIONS];
        let raw = unsafe {
            slice::from_raw_parts(partition_table_start, PARTITION_ENTRY_SIZE * MAX_PARTITIONS)
        };
        for (idx, entry) in entries.iter_mut().enumerate() {
            let offset = idx * PARTITION_ENTRY_SIZE;
            let partition_type = PartitionType::from_mbr_tag_byte(raw[offset + 4]);
            let lba = u32::from_le_bytes(raw[offset + 8..][..4].try_into().unwrap());
            let count = u32::from_le_bytes(raw[offset + 12..][..4].try_into().unwrap());
            *entry = PartitionTableEntry::new(partition_type, lba, count);
        }
        entries
    };

    // Locate the partition with the second stage on it
    let third_stage_partition_idx = partitions
        .iter()
        .enumerate()
        .find(|(_, e)| {
            e.partition_type == PartitionType::Unknown(BOOTLOADER_SECOND_STAGE_PARTITION_TYPE)
        })
        .unwrap()
        .0;
    let fat_partition = partitions.get(third_stage_partition_idx + 1).unwrap();
    writeln!(
        print::Writer,
        "Stage 2 partition idx: {third_stage_partition_idx} (LBA: 0x{0:x})\n",
        fat_partition.logical_block_address
    )
    .unwrap();
    writeln!(print::Writer, "Disk: {disk_number}\n").unwrap();
    writeln!(
        print::Writer,
        "Partition table start: 0x{:X}",
        partition_table_start as u16
    )
    .unwrap();

    let mut disk = disk_access::DiskAccess {
        disk_number,
        base_offset: u64::from(fat_partition.logical_block_address) * 512,
        current_offset: 0,
    };

    let disk_buffer: &mut disk_access::AlignedArrayBuffer<16384> = unsafe { &mut DISK_BUFFER };

    let mut fs = fat::FileSystem::parse(disk.clone());
    let stage2_len = load_file("boot-stage-2", STAGE_2_DST, &mut fs, &mut disk, disk_buffer);
    writeln!(
        print::Writer,
        "\nStage 2 loaded at {STAGE_2_DST:#p}. Size: 0x{stage2_len:x}"
    )
    .unwrap();

    let stage3_len = load_file("boot-stage-3", STAGE_3_DST, &mut fs, &mut disk, disk_buffer);
    writeln!(
        print::Writer,
        "\nStage 3 loaded at {STAGE_3_DST:#p}. Size: 0x{stage3_len:x}"
    )
    .unwrap();

    let max_width = 1280;
    let max_height = 720;
    let mut vesa_info = vesa::VesaInfo::query(disk_buffer).unwrap();
    let vesa_mode = vesa_info
        .get_best_mode(max_width, max_height)
        .unwrap()
        .unwrap();
    writeln!(
        print::Writer,
        "VESA MODE: {}x{} {:x}",
        vesa_mode.width,
        vesa_mode.height,
        vesa_mode.framebuffer_start
    )
    .unwrap();
    vesa_mode.enable().unwrap();

    let mut bios_info = BiosInfo {
        framebuffer: BiosFramebufferInfo {
            region: Region {
                start: vesa_mode.framebuffer_start.into(),
                len: u64::from(vesa_mode.height) * u64::from(vesa_mode.bytes_per_scanline),
            },
            width: vesa_mode.width,
            height: vesa_mode.height,
            bytes_per_pixel: vesa_mode.bytes_per_pixel,
            stride: vesa_mode.bytes_per_scanline / u16::from(vesa_mode.bytes_per_pixel),
            pixel_format: vesa_mode.pixel_format,
        },
        stage_4: Region { start: 0, len: 0 },
        kernel: Region { start: 0, len: 0 },
        ramdisk: Region { start: 0, len: 0 },
        config_file: Region { start: 0, len: 0 },
        last_used_addr: 0,
        memory_map_addr: 0,
        memory_map_len: 0,
    };

    protected_mode_jump_to_stage2(STAGE_2_DST, &mut bios_info);

    loop {
        fail::hlt()
    }
}

#[cfg(all(not(test), target_os = "none"))]
use core::arch::asm;

#[cfg(all(not(test), target_os = "none"))]
#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    writeln!(print::Writer, "PANIC!!!").unwrap();
    writeln!(print::Writer, "{}", info).unwrap();
    loop {
        unsafe {
            asm!("hlt");
        };
    }
}
