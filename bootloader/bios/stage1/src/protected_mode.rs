use core::{arch::asm, mem::size_of};
use x86_64::gdt::DescriptorFlags;

#[repr(C)]
struct GtdProtectedMode {
    zero: u64,
    code: u64,
    data: u64,
}

impl GtdProtectedMode {
    pub const fn new() -> Self {
        Self {
            zero: 0,
            code: DescriptorFlags::KERNEL_CODE32.bits(),
            data: DescriptorFlags::KERNEL_DATA.bits(),
        }
    }

    fn load(&'static self) {
        let pointer = GtdProtectedPointer {
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
struct GtdProtectedPointer {
    pub limit: u16,
    pub base: *const GtdProtectedMode,
}

static PROTECTED_GTD: GtdProtectedMode = GtdProtectedMode::new();

pub fn enter_unreal_mode() {
    // https://wiki.osdev.org/Unreal_Mode#Big_Unreal_Mode

    // Preserve DS (Data segment) and SS (Stack segment) registers
    let ds: u16;
    let ss: u16;
    unsafe {
        asm!("mov {0:x}, ds", out(reg) ds, options(nomem, nostack, preserves_flags));
        asm!("mov {0:x}, ss", out(reg) ss, options(nomem, nostack, preserves_flags));
        asm!("cli");

        // Load protected mode gtd
        PROTECTED_GTD.load();

        // Set protected mode bit
        let mut cr0: u32;
        {
            asm!(
                "mov {:e}, cr0",  // Store cr0
                out(reg) cr0,
            );
            let protected = cr0 | 0x1; // Set the protected mode bit
            asm!(
                "mov cr0, {:e}",
                in(reg) protected
            );
        }

        asm!(
            // Load the new GTD into segment registers,
            "mov {0:e}, 0x10",
            "mov ds, {0:e}",
            "mov ss, {0:e}", out(reg) _);

        asm!(
            // unset protected bit again
            "mov cr0, {0:e}",
            // Reset the segment registers
            "mov ds, {1:x}",
            "mov ss, {2:x}",
            // Re-enable interrupts
            "sti",
            in(reg) cr0,
            in(reg) ds,
            in(reg) ss
        );
    }

    // Restore DS and SS regs
    unsafe {
        asm!("mov ds, {0:x}", in(reg) ds, options(nostack, preserves_flags));
        asm!("mov ss, {0:x}", in(reg) ss, options(nostack, preserves_flags));
    }
}
