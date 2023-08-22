use bootloader_x86_64_bios::BiosInfo;
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
}

pub fn protected_mode_jump_to_stage2(entry_point: *const u8, bios_info: &mut BiosInfo) {
    unsafe {
        // Disable interrupts
        asm!("cli");

        // Enter protected mode
        let mut cr0: u32;
        asm!("mov {0:e}, cr0", out(reg) cr0);
        let protected_flags = cr0 | 0x1;
        asm!("mov cr0, {0:e}", in(reg) protected_flags);

        // Set up the stack
        asm!("and esp, 0xffffff00");
        // Push argument
        asm!("push {bios_info:e}", bios_info = in(reg) bios_info as *const _ as u32);
        // Push entry points
        asm!("push {entry_point:e}", entry_point = in(reg)entry_point as u32);

        // Long jump to 32bit sub below
        asm!("ljmp $0x8, $2f", "2:", options(att_syntax));

        asm!(
            // 32bit code
            ".code32",
            // Reload segment regs
            "mov {0}, 0x10",
            "mov ds, {0}",
            "mov es, {0}",
            "mov ss, {0}",

            // Jump to second stage
            "pop {1}",
            "call {1}",

            // Trap loop if stage2 returns
            "2:",
            "jmp 2b",
            out(reg)_,
            out(reg)_
        );
    }
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
