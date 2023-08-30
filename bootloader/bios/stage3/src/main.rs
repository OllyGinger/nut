#![no_std]
#![no_main]

use bootloader_x86_64_bios::BiosInfo;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(info: &mut BiosInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    //unsafe {
    //    bootloader_x86_64_common::logger::LOGGER
    //        .get()
    //        .map(|l| l.force_unlock())
    //};
    //log::error!("{info}");
    loop {
        unsafe { core::arch::asm!("cli; hlt") };
    }
}
