use core::{arch::asm, mem::size_of};

use x86_64::gdt::DescriptorFlags;

#[repr(C)]
pub struct GtdLongMode {
    zero: u64,
    code: u64,
    data: u64,
}

impl GtdLongMode {
    pub const fn new() -> Self {
        Self {
            zero: 0,
            code: DescriptorFlags::KERNEL_CODE64.bits(),
            data: DescriptorFlags::KERNEL_DATA64.bits(),
        }
    }

    pub fn load(&'static self) {
        let pointer = GtdLongPointer {
            base: self,
            limit: (3 * size_of::<u64>() - 1) as u16,
        };
        unsafe {
            asm!(
                "lgdt [{}]",
                in(reg) &pointer,
                options(readonly, nostack, preserves_flags)
            )
        }
    }
}

#[repr(C, packed(2))]
struct GtdLongPointer {
    pub limit: u16,
    pub base: *const GtdLongMode,
}

pub static LONG_GTD: GtdLongMode = GtdLongMode::new();
