#![no_std]
#![no_main]

use core::arch::asm;

#[no_mangle]
pub extern "C" fn print_char2(c: u8) {
    let ax = u16::from(c) | 0x0e00;
    unsafe {
        asm!("push bx", "mov bx, 0", "int 0x10", "pop bx", in("ax") ax);
    }
}

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(disk_number: u16, partition_table_start: *const u8) -> ! {
    print_char2(b'L');

    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}
