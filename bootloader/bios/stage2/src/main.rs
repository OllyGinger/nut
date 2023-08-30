#![no_std]
#![no_main]

use crate::screen::Writer;
use bootloader_x86_64_bios::BiosInfo;
use core::fmt::Write as _;

mod paging;
mod screen;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(bios_info: &mut BiosInfo) {
    screen::init(bios_info.framebuffer);

    // Writer.clear_screen();
    writeln!(Writer, "Third Stage").unwrap();

    paging::init();

    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

#[panic_handler]
#[cfg(not(test))]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
