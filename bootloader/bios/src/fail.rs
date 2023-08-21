use crate::print;
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

#[cold]
#[inline(never)]
#[no_mangle]
pub extern "C" fn fail(code: u8) -> ! {
    print::print_char(b'%');
    print::print_char(code);
    loop {
        hlt()
    }
}

pub fn hlt() {
    unsafe {
        asm!("hlt");
    }
}
