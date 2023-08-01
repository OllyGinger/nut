#![no_std]
#![no_main]

use core::fmt::Write;

mod print;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(disk_number: u16, partition_table_start: *const u8) -> ! {
    print::print_str("Hello world!\n");

    writeln!(print::Writer, "Disk: {disk_number}\n").unwrap();
    writeln!(
        print::Writer,
        "Table start: {}\n",
        partition_table_start as u16
    )
    .unwrap();

    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}
