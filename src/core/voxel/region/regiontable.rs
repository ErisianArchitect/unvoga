use crate::prelude::*;

use super::{regioncoord::RegionCoord, sectoroffset::SectorOffset, timestamp::Timestamp};

pub trait RegionTableItem: Default + Copy + Writeable + Readable {
    const OFFSET: u64;
}

impl RegionTableItem for Timestamp {
    const OFFSET: u64 = 0;
}

impl RegionTableItem for SectorOffset {
                        //  64-bit timestamps, offset is after timestamp table.
    const OFFSET: u64 = 1024*8;
}

pub struct RegionTable<T: RegionTableItem> {
    pub(super) table: Box<[T]>,
}

impl<T: RegionTableItem> RegionTable<T> {
    
    pub fn new() -> Self {
        Self {
            table: (0..1024).map(|_| T::default()).collect(),
        }
    }

    
    pub fn get(&self, x: i32, y: i32) -> T {
        let index = index2::<32>(x, y);
        self.table[index]
    }

    
    pub fn set(&mut self, x: i32, y: i32, value: T) -> T {
        let index = index2::<32>(x, y);
        self.table[index].swap(value)
    }

    
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.table.iter()
    }
}

impl<T: RegionTableItem> Writeable for RegionTable<T> {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> VoxelResult<u64> {
        self.table.iter().cloned().try_fold(0, |size, item| {
            Ok(size + item.write_to(writer)?)
        })
    }
}

impl<T: RegionTableItem> Readable for RegionTable<T> {
    fn read_from<R: std::io::Read>(reader: &mut R) -> VoxelResult<Self> {
        let collect: VoxelResult<Box<[T]>> = (0..1024).map(|_| T::read_from(reader)).collect();
        Ok(RegionTable {
            table: collect?
        })
    }
}

impl<T: RegionTableItem> std::ops::Index<RegionCoord> for RegionTable<T> {
    type Output = T;
    
    fn index(&self, index: RegionCoord) -> &Self::Output {
        &self.table[index.index()]
    }
}

impl<T: RegionTableItem> std::ops::IndexMut<RegionCoord> for RegionTable<T> {
    
    fn index_mut(&mut self, index: RegionCoord) -> &mut Self::Output {
        &mut self.table[index.index()]
    }
}

pub type TimestampTable = RegionTable<Timestamp>;
pub type OffsetTable = RegionTable<SectorOffset>;