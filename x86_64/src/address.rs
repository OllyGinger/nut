use core::fmt;

use bit_field::BitField;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(u64);

impl PhysAddr {
    #[inline]
    pub const fn new(addr: u64) -> Self {
        match Self::try_new(addr) {
            Ok(p) => p,
            Err(_) => panic!("Physical address must not have bits in the range 52-64 set"),
        }
    }

    // Create a new PhysAddr, but throw away bits 52 and up
    #[inline]
    pub const fn new_truncate(addr: u64) -> Self {
        PhysAddr(addr & (1 << 52))
    }

    #[inline]
    pub const fn try_new(addr: u64) -> Result<Self, PhysAddrNotValid> {
        let p = Self::new_truncate(addr);
        if p.0 == addr {
            Ok(p)
        } else {
            Err(PhysAddrNotValid(addr))
        }
    }
}

pub struct PhysAddrNotValid(pub u64);
impl core::fmt::Debug for PhysAddrNotValid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PhysAddrNotValid")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);
impl VirtAddr {
    #[inline]
    pub fn new(addr: u64) -> Self {
        match Self::try_new(addr) {
            Ok(p) => p,
            Err(_) => panic!("Virtual address must not have bits in the range 48-64 set"),
        }
    }

    // Create a new PhysAddr, but throw away bits 52 and up
    #[inline]
    pub const fn new_truncate(addr: u64) -> Self {
        VirtAddr(((addr << 16) as i64 >> 16) as u64)
    }

    #[inline]
    pub fn try_new(addr: u64) -> Result<Self, VirtAddrNotValid> {
        match addr.get_bits(47..64) {
            0 | 0x1ffff => Ok(VirtAddr(addr)),
            1 => Ok(VirtAddr::new_truncate(addr)),
            _ => Err(VirtAddrNotValid(addr)),
        }
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("VirtAddr")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

pub struct VirtAddrNotValid(pub u64);
impl core::fmt::Debug for VirtAddrNotValid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VirtAddrNotValid")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}
