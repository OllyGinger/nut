use bit_field::BitField;

use crate::privilege_level::PrivilegeLevel;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    #[inline]
    pub const fn new(index: u16, privilege_level: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (privilege_level as u16))
    }

    #[inline]
    pub fn index(self) -> u16 {
        self.0 >> 3
    }

    #[inline]
    pub fn privilege_level(self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.0.get_bits(0..2))
    }
}
