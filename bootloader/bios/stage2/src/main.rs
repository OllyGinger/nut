#![no_std]
#![no_main]

use crate::screen::Writer;
use bootloader_x86_64_bios::BiosInfo;
use core::arch::asm;
use core::fmt::Write as _;

mod gdt;
mod paging;
mod screen;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(bios_info: &mut BiosInfo) {
    screen::init(bios_info.framebuffer);

    // Writer.clear_screen();
    writeln!(Writer, "Stage 2").unwrap();

    paging::init();
    gdt::LONG_GTD.load();

    writeln!(Writer, "Paging init done, jumping to stage 3").unwrap();
    //enter_long_mode_and_jump_to_stage_3(bios_info);

    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

#[no_mangle]
pub fn enter_long_mode_and_jump_to_stage_3(info: &mut BiosInfo) {
    let _ = writeln!(Writer, "Paging init done, jumping to stage 4");
    unsafe {
        asm!(
            // align the stack
            "and esp, 0xffffff00",
            // push arguments (extended to 64 bit)
            "push 0",
            "push {info:e}",
            // push entry point address (extended to 64 bit)
            "push 0",
            "push {entry_point:e}",
            info = in(reg) info as *const _ as u32,
            entry_point = in(reg) info.stage_3.start as u32,
        );
        asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
        asm!(
            ".code64",

            // reload segment registers
            "mov {0}, 0x10",
            "mov ds, {0}",
            "mov es, {0}",
            "mov ss, {0}",

            // jump to 4th stage
            "pop rax",
            "pop rdi",
            "call rax",

            // enter endless loop in case 4th stage returns
            "2:",
            "jmp 2b",
            out(reg) _,
            out("rax") _,
            out("rdi") _,
        );
    }
}

#[panic_handler]
#[cfg(not(test))]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
