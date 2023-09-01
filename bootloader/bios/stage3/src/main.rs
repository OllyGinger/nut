#![no_std]
#![no_main]

use core::cmp;

use bootloader_x86_64_bios::{
    BiosFramebufferInfo, BiosInfo, E820MemoryRegion, FrameBufferInfo, LevelFilter, PixelFormat,
    PixelFormat2,
};

const GIGABYTE: u64 = 4096 * 512 * 512;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start(info: &mut BiosInfo) -> ! {
    let memory_map: &mut [E820MemoryRegion] = unsafe {
        core::slice::from_raw_parts_mut(
            info.memory_map_addr as *mut _,
            info.memory_map_len.try_into().unwrap(),
        )
    };

    memory_map.sort_unstable_by_key(|e| e.start_addr);
    let max_physical_address = {
        let max = memory_map
            .iter()
            .map(|r| r.start_addr + r.len)
            .max()
            .unwrap();
        // Don't use addresses higher than 4GiB as we're in protected mode
        // in the bootloader.
        cmp::min(max, 4 * GIGABYTE)
    };

    let framebuffer_info = init_logger(info.framebuffer, LevelFilter::Trace, true, false);
    log::info!("4th Stage");

    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

fn init_logger(
    info: BiosFramebufferInfo,
    log_level: LevelFilter,
    frame_buffer_logger_status: bool,
    serial_logger_status: bool,
) -> FrameBufferInfo {
    let framebuffer_info = FrameBufferInfo {
        byte_len: info.region.len.try_into().unwrap(),
        width: info.width.into(),
        height: info.height.into(),
        pixel_format: match info.pixel_format {
            bootloader_x86_64_bios::PixelFormat::Rgb => PixelFormat2::Rgb,
            bootloader_x86_64_bios::PixelFormat::Bgr => PixelFormat2::Bgr,
            bootloader_x86_64_bios::PixelFormat::Unknown {
                red_position,
                green_position,
                blue_position,
            } => PixelFormat2::Unknown {
                red_position,
                green_position,
                blue_position,
            },
        },
        bytes_per_pixel: info.bytes_per_pixel.into(),
        stride: info.stride.into(),
    };

    let framebuffer = unsafe {
        core::slice::from_raw_parts_mut(
            info.region.start as *mut u8,
            info.region.len.try_into().unwrap(),
        )
    };

    bootloader_x86_64_bios::init_logger(
        framebuffer,
        framebuffer_info,
        log_level,
        frame_buffer_logger_status,
    );

    framebuffer_info
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
