use core::arch::asm;

use bootloader_x86_64_bios::racy_cell::RacyCell;

// https://wiki.osdev.org/Paging

static LEVEL4_TABLE: RacyCell<PageTable> = RacyCell::new(PageTable::empty());
static LEVEL3_TABLE: RacyCell<PageTable> = RacyCell::new(PageTable::empty());
static LEVEL2_TABLE: RacyCell<[PageTable; 10]> = RacyCell::new([PageTable::empty(); 10]);

pub fn init() {
    create_mappings();
    enable_paging();
}

pub fn create_mappings() {
    let l4 = unsafe { LEVEL4_TABLE.get_mut() };
    let l3 = unsafe { LEVEL3_TABLE.get_mut() };
    let l2 = unsafe { LEVEL2_TABLE.get_mut() };
    let common_flags = 0b11; // PRESENT | WRITABLE
    l4.entries[0] = (l3 as *mut PageTable as u64) | common_flags;
    for (i, l2entry) in l2.iter_mut().enumerate() {
        l3.entries[i] = (l2entry as *mut PageTable as u64) | common_flags;
        let offset = u64::try_from(i).unwrap() * 1024 * 1024 * 1024;
        for (j, entry) in l2entry.entries.iter_mut().enumerate() {
            *entry =
                (offset + u64::try_from(j).unwrap() * (2 * 1024 * 1024)) | common_flags | (1 << 7);
        }
    }
}

pub fn enable_paging() {
    unsafe {
        // Set the page table
        let level4_table = LEVEL4_TABLE.get_mut() as *mut PageTable;
        asm!(
            "mov cr3, {0:e}",
            in(reg) level4_table
        );

        // Enable the PAE (Physical Address Extension)
        asm!(
            "mov eax, cr4",
            "or eax, (1 << 5)",
            "mov cr4, eax",
            out("eax") _
        );

        // Enable long mode
        asm!(
            // Read the MSR (Model specific register) - IA32_EFER - Extended Feature Enables
            "mov ecx, 0xC0000080",
            // Read the MSR specified by ECX into EAX
            "rdmsr",
            // Set 8th bit - IA-32e Mode Enable: IA32_EFER.LME (R/W) - Enables IA-32e mode operation.
            "or eax, (1<<8)",
            // Write the MSR back
            "wrmsr",
            out("eax")_,
            out("ecx")_
        );

        // Enable paging and protected mode
        asm!(
            "mov eax, cr0",
            "or eax, 1<<31",
            "mov cr0, eax",
            out("eax") _
        );
    }
}

#[derive(Clone, Copy)]
#[repr(align(4096))]
struct PageTable {
    pub entries: [u64; 512],
}

impl PageTable {
    pub const fn empty() -> Self {
        Self { entries: [0; 512] }
    }
}
