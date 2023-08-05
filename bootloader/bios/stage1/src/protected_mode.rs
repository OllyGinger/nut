use core::{arch::asm, mem::size_of};
use x86_64::gdt::{Descriptor, DescriptorFlags, GlobalDescriptorTable};

struct GtdProtectedMode {
    zero: u64,
    code: u64,
    data: u64,
}

impl GtdProtectedMode {
    pub const fn new() -> Self {
        Self {
            zero: 0,
            code: DescriptorFlags::KERNEL_CODE64.bits(),
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

struct GtdProtectedPointer {
    pub limit: u16,
    pub base: *const GtdProtectedMode,
}

static PROTECTED_GTD: GtdProtectedMode = GtdProtectedMode::new();

pub fn enter_unreal_mode() {
    // Preserve DS (Data segment) and SS (Stack segment) registers
    let ds: u16;
    let ss: u16;
    unsafe {
        asm!(
            "mov {ds:x}, ds", 
            "mov {ss:x}, ss", 
            "cli",
            ds = out(reg) ds, 
            ss = out(reg) ss,
            options(nomem, nostack, preserves_flags));

        // Load protected mode gtd
        PROTECTED_GTD.load();

        // Set protected mode bit
        let mut cr0: u32;
        {
            asm!(
                "mov {:e}, cr0",  // Store cr0
                out(reg) cr0,
                options(nostack, nomem, preserves_flags)
            );
            let protected = cr0 | 0x1;
            asm!(
                "mov cr0, {:e}",
                in(reg) protected,
                options(nostack, nomem, preserves_flags)
            );
        }

        asm!(
            // Load the new GTD into segment registers,
            "mov {0}, 0x10",
            "mov ds, {0}",
            "mov ss, {0}",
            // unset protected bit again
            "mov cr0, {1:e}",
            // Reset the segment registers
            "mov ds, {2:x}",
            "mov ss, {3:x}",
            // Re-enable interrupts
            "sti",
            out(reg) _,
            in(reg) cr0,
            in(reg) ds,
            in(reg) ss,
        );
    }

    // Restore DS and SS regs
    unsafe {
        asm!("mov ds, {0:x}", in(reg) ds, options(nostack, preserves_flags));
        asm!("mov ss, {0:x}", in(reg) ss, options(nostack, preserves_flags));
    }
}
