#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PartitionType {
    None,
    Unknown(u8),
    Fat12(u8),
    Fat16(u8),
    Fat32(u8),
}

impl PartitionType {
    pub fn from_mbr_tag_byte(tag: u8) -> PartitionType {
        match tag {
            0x0 => PartitionType::None,
            0x01 => PartitionType::Fat12(tag),
            0x04 | 0x06 | 0x0e => PartitionType::Fat16(tag),
            0x0b | 0x0c | 0x1b | 0x1c => PartitionType::Fat32(tag),
            _ => PartitionType::Unknown(tag),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PartitionTableEntry {
    // What type f partition
    pub partition_type: PartitionType,

    // Index of the first block of this entry
    pub logical_block_address: u32,

    // Sector count
    pub sector_count: u32,
}

impl PartitionTableEntry {
    pub fn new(
        partition_type: PartitionType,
        logical_block_address: u32,
        sector_count: u32,
    ) -> PartitionTableEntry {
        PartitionTableEntry {
            partition_type,
            logical_block_address,
            sector_count,
        }
    }

    pub fn empty() -> PartitionTableEntry {
        PartitionTableEntry::new(PartitionType::None, 0, 0)
    }
}
