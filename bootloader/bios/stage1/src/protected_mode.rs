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
    unsafe {
        // Disable interrupts
        asm!("cli");

        asm!("push ds");
        asm!("push ss");
        asm!("push fs");
        asm!("push gs");

        // Load the GDT register
        PROTECTED_GTD.load();

        // Enter protected mode
        let mut cr0: u32;
        asm!("mov {0:e}, cr0", out(reg) cr0);
        let protected_flags = cr0 | 0x1; // set protected mode bit
        asm!("mov cr0, {0:e}", in(reg) protected_flags);

        // --- We're in protected mode ---
        asm!("mov bx, 0x10"); // Selector to use
        asm!("mov ds, bx");
        asm!("mov ss, bx");
        asm!("mov fs, bx");
        asm!("mov gs, bx");

        // Exit protected mode
        asm!("mov cr0, {0:e}", in(reg) cr0);

        // Get back old segments
        asm!("pop gs");
        asm!("pop fs");
        asm!("pop ss");
        asm!("pop ds");

        // Enable interrupts
        asm!("sti");

        asm!("mov bx, 0x0f01");
        asm!("mov eax, 0xb8f00");
        asm!("mov word ptr ds:[eax], bx");
    }

    /*// Preserve DS (Data segment) and SS (Stack segment) registers
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
                options(nomem, nostack, preserves_flags)
            );
            let protected = cr0 | 0x1; // Set the protected mode bit
            asm!(
                "mov cr0, {:e}",
                in(reg) protected,
                options(nostack, preserves_flags)
            );
        }

        asm!(
            // Load the new GTD into segment registers,
            "mov {0}, 0x10",
            "mov ds, {0}",
            "mov ss, {0}", out(reg) _);

        // unset protected bit again
        asm!("mov cr0, {:e}", in(reg) cr0, options(nostack, preserves_flags));

        // Reset the segment registers
        asm!("mov ds, {0:x}", in(reg) ds, options(nostack, preserves_flags));
        asm!("mov ss, {0:x}", in(reg) ss, options(nostack, preserves_flags));

        asm!(
            // Re-enable interrupts
            "sti",
        )
    }

    // Restore DS and SS regs
    unsafe {
        asm!("mov ds, {0:x}", in(reg) ds, options(nostack, preserves_flags));
        asm!("mov ss, {0:x}", in(reg) ss, options(nostack, preserves_flags));
    }*/
}

#[no_mangle]
pub unsafe fn copy_to_protected_mode(target: *mut u8, bytes: &[u8]) {
    for (offset, byte) in bytes.iter().enumerate() {
        let dst = target.wrapping_add(offset);
        // we need to do the write in inline assembly because the compiler
        // seems to truncate the address
        unsafe {
            asm!("mov [{}], {}", in(reg) dst, in(reg_byte) *byte, options(nostack, preserves_flags))
        };
        assert_eq!(read_from_protected_mode(dst), *byte);
    }
}

#[no_mangle]
pub unsafe fn read_from_protected_mode(ptr: *mut u8) -> u8 {
    let res;
    // we need to do the read in inline assembly because the compiler
    // seems to truncate the address
    unsafe {
        asm!("mov {}, [{}]", out(reg_byte) res, in(reg) ptr, options(pure, readonly, nostack, preserves_flags))
    };
    res
}
