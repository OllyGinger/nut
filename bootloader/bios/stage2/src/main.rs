#![no_std]
#![no_main]

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() {
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

#[panic_handler]
#[cfg(not(test))]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
