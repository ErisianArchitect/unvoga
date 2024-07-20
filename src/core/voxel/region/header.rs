use std::io::{Read, Seek};

use crate::prelude::{Readable, Writeable};

use super::{regiontable::{OffsetTable, TimestampTable}, sectoroffset::SectorOffset, timestamp::Timestamp};

pub struct RegionHeader {
    pub(super) timestamps: TimestampTable,
    pub(super) offsets: OffsetTable,
}

impl RegionHeader {
    // sizeof(Timestamp) * 1024 + sizeof(Offset) * 1024
    pub const HEADER_SIZE: u64 = 12288;
    pub fn new() -> Self {
        Self {
            timestamps: TimestampTable::new(),
            offsets: OffsetTable::new()
        }
    }
    
    
    pub fn get_timestamp(&self, x: i32, y: i32) -> Timestamp {
        self.timestamps.get(x, y)
    }

    
    pub fn set_timestamp<T: Into<Timestamp>>(&mut self, x: i32, y: i32, timestamp: T) -> Timestamp {
        self.timestamps.set(x, y, timestamp.into())
    }

    
    pub fn get_offset(&self, x: i32, y: i32) -> SectorOffset {
        self.offsets.get(x, y)
    }

    
    pub fn set_offset(&mut self, x: i32, y: i32, offset: SectorOffset) -> SectorOffset {
        self.offsets.set(x, y, offset)
    }
}

impl Readable for RegionHeader {
    fn read_from<R: Read>(reader: &mut R) -> crate::prelude::VoxelResult<Self> {
        Ok(Self {
            timestamps: TimestampTable::read_from(reader)?,
            offsets: OffsetTable::read_from(reader)?
        })
    }
}

impl Writeable for RegionHeader {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> crate::prelude::VoxelResult<u64> {
        Ok(
            self.timestamps.write_to(writer)? + 
            self.offsets.write_to(writer)?
        )
    }
}