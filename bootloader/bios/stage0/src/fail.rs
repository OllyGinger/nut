// This is basically a copy of bootloader_x86_64_bios::print, but we need to make sure
// this doesn't go over 512bytes so we don't import reference that crate

use core::arch::asm;

pub trait UnwrapOrFail {
    type Out;

    fn unwrap_or_fail(self, code: u8) -> Self::Out;
}

impl<T> UnwrapOrFail for Option<T> {
    type Out = T;

    fn unwrap_or_fail(self, code: u8) -> Self::Out {
        match self {
            Some(v) => v,
            None => fail(code),
        }
    }
}

impl<T, E> UnwrapOrFail for Result<T, E> {
    type Out = T;

    fn unwrap_or_fail(self, code: u8) -> Self::Out {
        match self {
            Ok(v) => v,
            Err(_) => fail(code),
        }
    }
}

#[no_mangle]
pub extern "C" fn print_char(c: u8) {
    let ax = u16::from(c) | 0x0e00;
    unsafe {
        asm!("push bx", "mov bx, 0", "int 0x10", "pop bx", in("ax") ax);
    }
}

#[cold]
#[inline(never)]
#[no_mangle]
pub extern "C" fn fail(code: u8) -> ! {
    print_char(b'F');
    print_char(code);
    loop {
        hlt()
    }
}

#[cold]
#[inline(never)]
#[no_mangle]
pub extern "C" fn fail2(code: u8) -> ! {
    print_char(b'#');
    print_char(code);
    loop {
        print_char(b'#');
    }
}

fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

#[panic_handler]
#[cfg(not(test))]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    fail(b'P');
}
