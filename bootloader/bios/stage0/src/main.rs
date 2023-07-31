#![no_std]
#![no_main]

use core::{arch::global_asm, slice};

use fail::UnwrapOrFail;

mod dap;
mod fail;
mod mbr;

global_asm!(include_str!("boot.s"));

extern "C" {
    static _partition_table: u8;
    static _stage1_start: u8;
}

unsafe fn partition_table_ptr() -> *const u8 {
    unsafe { &_partition_table }
}

fn stage1_start() -> *const () {
    let ptr: *const u8 = unsafe { &_stage1_start };
    ptr as *const ()
}

#[no_mangle]
pub extern "C" fn stage0(disk_number: u16) {
    let partition_table = unsafe { slice::from_raw_parts(partition_table_ptr(), 16 * 4) };
    let stage1_partition = mbr::get_partition(partition_table, 0);

    // Load the address of stage1
    let stage1_entry_point_address = stage1_start() as u32;
    let mut start_lba: u64 = stage1_partition.logical_block_address.into();
    let mut sector_count = stage1_partition.sector_count;
    let mut target_address = stage1_entry_point_address;

    loop {
        let sectors = u32::min(sector_count, 32) as u16;
        let dap = dap::DiskAddressPacket::from_lba(
            start_lba,
            sectors,
            (target_address & 0b1111) as u16,
            (target_address >> 4).try_into().unwrap_or_fail(b'a'),
        );
        unsafe { dap.perform_load(disk_number) }

        start_lba += u64::from(sectors);
        sector_count -= u32::from(sectors);
        target_address += u32::from(sectors) * 512;

        if sector_count == 0 {
            break;
        }
    }

    // Jump to the second stage
    let stage1_entry_point: extern "C" fn(disk_number: u16, partition_table_start: *const u8) =
        unsafe { core::mem::transmute(stage1_entry_point_address as *const ()) };
    let partition_table_start = unsafe { partition_table_ptr() };

    // Call it
    stage1_entry_point(disk_number, partition_table_start);

    fail::fail(b'R');
}
