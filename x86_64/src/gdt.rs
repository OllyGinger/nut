use bitflags::bitflags;

use crate::{
    address::VirtAddr, instructions::tables::lgdt, privilege_level::PrivilegeLevel,
    registers::segmentation::SegmentSelector,
};

#[derive(Clone, Debug)]
pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

impl Descriptor {
    #[inline]
    pub const fn kernel_code_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::KERNEL_CODE64.bits())
    }
    #[inline]
    pub const fn kernel_data_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::KERNEL_DATA.bits())
    }
    #[inline]
    pub const fn user_code_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::USER_CODE64.bits())
    }
    #[inline]
    pub const fn user_data_segment() -> Descriptor {
        Descriptor::UserSegment(DescriptorFlags::USER_DATA.bits())
    }
}
#[derive(Copy, Clone, Debug)]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: VirtAddr,
}

#[derive(Clone, Debug)]
pub struct GlobalDescriptorTable {
    table: [u64; 8],
    len: usize,
}

impl GlobalDescriptorTable {
    #[inline]
    pub const fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; 8],
            len: 1,
        }
    }

    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => {
                if self.len > self.table.len().saturating_sub(1) {
                    panic!("GDT Full");
                }
                self.push(value)
            }
            Descriptor::SystemSegment(value_low, value_high) => {
                if self.len > self.table.len().saturating_sub(2) {
                    panic!("GDT Full - SystemSegment requires two slots");
                }
                let idx = self.push(value_low);
                self.push(value_high);
                idx
            }
        };

        let requested_privilege_level = match entry {
            Descriptor::UserSegment(value) => {
                if DescriptorFlags::from_bits_truncate(value)
                    .contains(DescriptorFlags::DESCRIPTOR_PRIVILEGE_LEVEL)
                {
                    PrivilegeLevel::Ring3
                } else {
                    PrivilegeLevel::Ring0
                }
            }
            Descriptor::SystemSegment(_, _) => PrivilegeLevel::Ring0,
        };

        SegmentSelector::new(index as u16, requested_privilege_level)
    }

    fn push(&mut self, value: u64) -> usize {
        let idx = self.len;
        self.table[idx] = value;
        self.len += 1;
        idx
    }

    pub fn load(&'static self) {
        unsafe { self.load_unsafe() };
    }

    pub unsafe fn load_unsafe(&self) {
        unsafe {
            lgdt(&self.pointer());
        }
    }

    fn pointer(&self) -> DescriptorTablePointer {
        use core::mem::size_of;
        DescriptorTablePointer {
            base: VirtAddr::new(self.table.as_ptr() as u64),
            limit: (self.len * size_of::<u64>() - 1) as u16,
        }
    }
}

bitflags! {
    pub struct DescriptorFlags : u64{
        // Bits `0..=23` of the base address field (not used in 64-bit except FS and GS)
        const BASE_ADDRESS_0_23 = 0xFF_FFFF << 16;
        // Bits `0..=31` of the base address field (not used in 64-bit except FS and GS)
        const BASE_ADDRESS_24_31 = 0xFF << 56;

        // Bits `0..=15` of the segment limit field (not used in 64-bit)
        const SEGMENT_LIMIT_0_15 = 0xFFFF;
        // Bits `16..=19` of the segment limit field (not used in 64-bit)
        const SEGMENT_LIMIT_16_19 = 0xF << 48;

        // Granularity - If set then limit is in units of 4096-byte pages - max of 2^32 bytes.
        // (Ignored in 64-bit mode)
        const GRANULARITY = 0x1 << 55;

        // Default operand size. If clear this is a 16-bit code segment, else it's a 32-bit segment.
        // If Self::LONG_MODE is set then this should be clear
        const DEFAULT_OP_SIZE = 0x1 << 54;

        // Long mode - If this is set (Self::DEFAULT_OP_SIZE must be clear) this is a 64-bit segment.
        const LONG_MODE = 0x1 << 53;

        // Available - For use by the operating system.
        const AVAILABLE = 0x1 << 52;

        // Present - Must be set for any segment. Causes a 'segment not present' exception if not set.
        const PRESENT = 0x1 << 47;

        // Descriptor privilege level. The privilege level (ring) required to access this descriptor.
        const DESCRIPTOR_PRIVILEGE_LEVEL = 0x3 << 45;

        // System segment. If clear this is a system segment, if set this is a code/data segment.
        const USER_SEGMENT = 0x1 << 44;

        // Set this for code segments, and unset for data segments.
        const EXECUTABLE = 0x1 << 43;

        // Conforming;
        //    * For code segments setting this bit sets the segment as "conforming", meaning
        //      code can be called from less-privileged levels.
        //    * For 32-bit data segments, sets the segment as "expand down".
        //    * In 64-bit mode this is ignored for data segments.
        const CONFORMING = 0x1 << 42;

        // Writable. For 32-bit data segments, setting this sets the segment to writable. For
        // code segments, setting this sets the segment to readable. 64-bit mode ignores this.
        const WRITABLE = 0x1 << 41;

        // This is set by the processor if this segment has been accessed. Should be cleared
        // by software.
        const ACCESSED = 0x1 << 40;
    }
}

impl DescriptorFlags {
    #[allow(unused)]
    const COMMON_FLAGS: Self = Self::from_bits_truncate(
        Self::USER_SEGMENT.bits()
            | Self::PRESENT.bits()
            | Self::WRITABLE.bits()
            | Self::ACCESSED.bits()
            | Self::SEGMENT_LIMIT_0_15.bits()
            | Self::SEGMENT_LIMIT_16_19.bits()
            | Self::GRANULARITY.bits(),
    );

    // Kernel data segment
    #[allow(unused)]
    pub const KERNEL_DATA: Self =
        Self::from_bits_truncate(Self::COMMON_FLAGS.bits() | Self::DEFAULT_OP_SIZE.bits());
    // 32-bit kernel code segment
    #[allow(unused)]
    pub const KERNEL_CODE32: Self = Self::from_bits_truncate(
        Self::COMMON_FLAGS.bits() | Self::DEFAULT_OP_SIZE.bits() | Self::EXECUTABLE.bits(),
    );
    // 64-bit kernel code segment
    #[allow(unused)]
    pub const KERNEL_CODE64: Self =
        Self::from_bits_truncate(Self::KERNEL_CODE32.bits() | Self::LONG_MODE.bits());

    // Usermode data
    #[allow(unused)]
    pub const USER_DATA: Self = Self::from_bits_truncate(
        Self::KERNEL_DATA.bits() | Self::DESCRIPTOR_PRIVILEGE_LEVEL.bits(),
    );
    // 32-bit user code segment
    #[allow(unused)]
    pub const USER_CODE32: Self = Self::from_bits_truncate(
        Self::KERNEL_CODE32.bits() | Self::DESCRIPTOR_PRIVILEGE_LEVEL.bits(),
    );
    // 64-bit user code segment
    #[allow(unused)]
    pub const USER_CODE64: Self = Self::from_bits_truncate(
        Self::KERNEL_CODE64.bits() | Self::DESCRIPTOR_PRIVILEGE_LEVEL.bits(),
    );
}
