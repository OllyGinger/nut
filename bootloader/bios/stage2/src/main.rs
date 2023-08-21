#![no_std]
#![no_main]

use bootloader_x86_64_bios::print;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    print::print_str("Starting Stage 2...\n");

    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

#[panic_handler]
#[cfg(not(test))]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
