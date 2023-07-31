use super::fail::UnwrapOrFail;

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct Partition {
    // Is the partition bootable
    pub(crate) bootable: bool,

    // Type of partition
    pub(crate) partition_type: u8,

    // Index to the first block of this partition (LBA)
    pub(crate) logical_block_address: u32,

    // Total number of sectors
    pub(crate) sector_count: u32,
}

impl Partition {
    pub fn new(
        bootable: bool,
        partition_type: u8,
        logical_block_address: u32,
        sector_count: u32,
    ) -> Partition {
        Partition {
            bootable,
            partition_type,
            logical_block_address,
            sector_count,
        }
    }
}

pub(crate) fn get_partition(partitions_ptr: &[u8], index: usize) -> Partition {
    const PARTITION_ENTRY_SIZE: usize = 16;

    let offset = index * PARTITION_ENTRY_SIZE;
    let buffer = partitions_ptr.get(offset..).unwrap_or_fail(b'c');

    let bootable_ptr = *buffer.first().unwrap_or_fail(b'd');
    let is_bootable = bootable_ptr == 0x80;
    let partition_type = *buffer.get(4).unwrap_or_fail(b'e');
    let lba = u32::from_le_bytes(
        buffer
            .get(8..)
            .and_then(|s| s.get(..4))
            .and_then(|s| s.try_into().ok())
            .unwrap_or_fail(b'e'),
    );
    let count = u32::from_le_bytes(
        buffer
            .get(12..)
            .and_then(|s| s.get(..4))
            .and_then(|s| s.try_into().ok())
            .unwrap_or_fail(b'f'),
    );

    Partition::new(is_bootable, partition_type, lba, count)
}
