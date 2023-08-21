use core::arch::asm;

use crate::print;
use core::fmt::Write;

pub enum SeekFrom {
    Start(u64),
}

pub trait AlignedBuffer {
    fn slice(&self) -> &[u8];
    fn slice_mut(&mut self) -> &mut [u8];
}

#[repr(align(2))]
pub struct AlignedArrayBuffer<const LEN: usize> {
    pub buffer: [u8; LEN],
}

impl<const LEN: usize> AlignedBuffer for AlignedArrayBuffer<LEN> {
    fn slice(&self) -> &[u8] {
        &self.buffer[..]
    }

    fn slice_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[..]
    }
}

pub trait Read {
    unsafe fn read_exact(&mut self, len: usize) -> &[u8];
    fn read_exact_into(&mut self, len: usize, buf: &mut dyn AlignedBuffer);
}

impl Read for DiskAccess {
    unsafe fn read_exact(&mut self, len: usize) -> &[u8] {
        let current_sector_offset = usize::try_from(self.current_offset % 512).unwrap();

        static mut TMP_BUF: AlignedArrayBuffer<1024> = AlignedArrayBuffer {
            buffer: [0; 512 * 2],
        };
        let buf = unsafe { &mut TMP_BUF };
        assert!(current_sector_offset + len <= buf.buffer.len());

        self.read_exact_into(buf.buffer.len(), buf);

        &buf.buffer[current_sector_offset..][..len]
    }

    fn read_exact_into(&mut self, len: usize, buf: &mut dyn AlignedBuffer) {
        assert_eq!(len % 512, 0);
        let buf = &mut buf.slice_mut()[..len];

        let end_addr = self.base_offset + self.current_offset + u64::try_from(buf.len()).unwrap();
        let mut start_lba = (self.base_offset + self.current_offset) / 512;
        let end_lba = (end_addr - 1) / 512;

        let mut number_of_sectors = end_lba + 1 - start_lba;
        let mut target_addr = buf.as_ptr_range().start as u32;

        writeln!(
            print::Writer,
            "---- START: 0x{:x} - END: 0x{:x}",
            target_addr,
            end_addr
        )
        .unwrap();

        loop {
            let sectors = u64::min(number_of_sectors, 32) as u16;
            let dap = DiskAccessPacket::from_lba(
                start_lba,
                sectors,
                (target_addr & 0b1111) as u16,
                (target_addr >> 4).try_into().unwrap(),
            );

            writeln!(print::Writer, "DAP: {:?}", dap).unwrap();
            unsafe {
                dap.read_disk(self.disk_number);
            }

            start_lba += u64::from(sectors);
            number_of_sectors -= u64::from(sectors);
            target_addr += u32::from(sectors) * 512;

            if number_of_sectors == 0 {
                break;
            }
        }

        self.current_offset = end_addr;
    }
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> u64;
}

impl Seek for DiskAccess {
    fn seek(&mut self, pos: SeekFrom) -> u64 {
        match pos {
            SeekFrom::Start(offset) => {
                self.current_offset = offset;
                self.current_offset
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DiskAccess {
    pub disk_number: u16,
    pub base_offset: u64,
    pub current_offset: u64,
}

// Packet that is used to request data from a drive via the bios
// https://en.wikipedia.org/wiki/INT_13H#INT_13h_AH=42h:_Extended_Read_Sectors_From_Drive
// https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
#[repr(C, packed)]
struct DiskAccessPacket {
    // The size of the disk access packet (should usually be 0x10)
    packet_size: u8,
    // Unused. Should always zero
    zero: u8,
    // Number of sectors to read
    num_sectors: u16,
    // Offset into the memory buffer segment
    offset: u16,
    // Segment of memory buffer
    segment: u16,
    // Start of logical block address to read to/write from
    start_lba: u64,
}

impl DiskAccessPacket {
    pub fn from_lba(
        start_lba: u64,
        number_of_sectors: u16,
        target_addr: u16,
        target_addr_segment: u16,
    ) -> DiskAccessPacket {
        DiskAccessPacket {
            packet_size: 0x10,
            zero: 0,
            num_sectors: number_of_sectors,
            offset: target_addr,
            segment: target_addr_segment,
            start_lba,
        }
    }
    unsafe fn read_disk(&self, disk_number: u16) {
        let self_ptr = self as *const Self as u16;
        unsafe {
            asm!(
                "push 0x7a",    // Push the error code of 'z' that will signify an error while reading.
                                // The CPU will set the carry flag if the read fails
                "mov {1:x}, si", // Save the SI register
                "mov si, {0:x}",// Set SI to the pointer of the DAP in memory (Self)
                "int 0x13",     // 0x13 interrupt
                "jc fail",      // call 'fail' if the read fails (carry bit set)
                "pop si",       // Pop the error code
                "mov si, {1:x}",// Restore the old SI value
                in(reg) self_ptr,
                out(reg) _,
                in("ax") 0x4200u16, // 0x42 is `Extended Read Sectors From Drive`
                in("dx") disk_number,// Disk number we want to read
            );
        }
    }
}
