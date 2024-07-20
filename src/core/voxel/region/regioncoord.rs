use super::{regiontable::RegionTableItem, sectoroffset::SectorOffset, timestamp::Timestamp};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegionCoord(u16);

impl RegionCoord {
    
    pub fn new(x: i32, z: i32) -> Self {
        RegionCoord((x & 31) as u16 | ((z & 31) as u16) << 5)
    }

    
    pub fn x(self) -> i32 {
        (self.0 & 0x31) as i32
    }

    
    pub fn z(self) -> i32 {
        (self.0 & 0x31) as i32
    }

    
    pub fn index(self) -> usize {
        self.0 as usize
    }

    
    pub fn sector_offset(self) -> u64 {
        SectorOffset::OFFSET + 4 * self.0 as u64
    }

    
    pub fn timestamp_offset(self) -> u64 {
        Timestamp::OFFSET + 8 * self.0 as u64
    }
}

impl Into<(i32, i32)> for RegionCoord {
    
    fn into(self) -> (i32, i32) {
        (self.x(), self.z())
    }
}

impl From<(i32, i32)> for RegionCoord {
    
    fn from(value: (i32, i32)) -> Self {
        Self::new(value.0, value.1)
    }
}