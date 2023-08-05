use core::arch::asm;

use crate::gdt::DescriptorTablePointer;

// Load a Global Descriptor Table

pub unsafe fn lgdt(gtd: &DescriptorTablePointer) {
    unsafe {
        asm!("lgdt [{}]", in(reg) gtd, options(readonly, nostack, preserves_flags));
    }
}
