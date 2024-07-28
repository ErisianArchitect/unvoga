#![allow(unused)]
use std::ops::Range;

use itertools::Itertools;

use crate::core::error::*;

use super::regiontable::OffsetTable;
use super::sectoroffset::BlockSize;
use super::sectoroffset::SectorOffset;

pub struct SectorManager {
    unused: Vec<ManagedSector>,
    end_sector: ManagedSector,
}

impl SectorManager {
    /// The max size representable by the [BlockSize] type.
    pub const MAX_SECTOR_SIZE: u32 = BlockSize::MAX_BLOCK_COUNT as u32 * 4096;

    
    pub fn new() -> Self {
        Self {
            unused: Vec::new(),
            end_sector: ManagedSector::new(3, u32::MAX),
        }
    }

    pub fn from_sector_table(table: &OffsetTable) -> Self {
        let mut filtered_sectors = table.iter().cloned()
            .map(ManagedSector::from)
            .filter(|sector| sector.not_empty())
            .collect_vec();
        filtered_sectors.sort();
        let initial_state = (
            Vec::<ManagedSector>::new(),
            ManagedSector::HEADER,
        );
        let (
            unused,
            end_sector
        ) = filtered_sectors.into_iter()
            .fold(initial_state, |(mut unused_sectors, previous), sector| {
                if let Some(_) = previous.gap(sector) {
                    unused_sectors.push(ManagedSector::new(
                        previous.end,
                        sector.start
                    ));
                }
                (
                    unused_sectors,
                    sector
                )
            });
        Self {
            unused,
            end_sector
        }
    }

    /// Attempts to allocate a sector. Panics if `blocks_required` exceed 8033.
    pub fn allocate(&mut self, blocks_required: u16) -> SectorOffset {
        if blocks_required > BlockSize::MAX_BLOCK_COUNT {
            panic!("Requested size exceeds maximum.");
        }
        let block_size = BlockSize::required(blocks_required);
        let block_count = block_size.block_count();
        // Find sector with needed size
        self.unused.iter().cloned().enumerate()
            .find(|(_, sector)| sector.size() >= (block_count as u32))
            .and_then(|(index, sector)| {
                let (result, new_sector) = sector.split_left(block_size.block_count() as u32);
                if new_sector.is_empty() {
                    self.unused.swap_remove(index);
                } else {
                    self.unused[index] = new_sector;
                }
                Some(SectorOffset::new(block_size, result.start))
            })
            .or_else(|| {
                self.end_sector.allocate(block_size)
            }).expect("Allocation failed.")
    }

    pub fn deallocate<S: Into<ManagedSector>>(&mut self, sector: S) {
        let sector: ManagedSector = sector.into();
        if sector.size() == 0 {
            return;
        }
        let mut freed_sector = ManagedSector::from(sector);
        let mut left: Option<usize> = None;
        let mut right: Option<usize> = None;
        self.unused.iter()
            .cloned()
            .enumerate()
            // .find_map for early return
            .find_map(|(index, sector)| {
                match (left, right) {
                    (None, Some(_)) => {
                        if sector.end != freed_sector.start {
                            return None;
                        }
                        left = Some(index);
                        Some(())
                    }
                    (Some(_), None) => {
                        if freed_sector.end != sector.start {
                            return None;
                        }
                        right = Some(index);
                        Some(())
                    }
                    (None, None) => {
                        if sector.end == freed_sector.start {
                            left = Some(index);
                        } else if freed_sector.end == sector.start {
                            right = Some(index);
                        }
                        None
                    }
                    _ => Some(())
                }
            });
        match (left, right) {
            (Some(left), Some(right)) => {
                freed_sector.absorb(
                    self.unused.swap_remove(right.max(left))
                );
                freed_sector.absorb(
                    self.unused.swap_remove(left.min(right))
                );
            }
            (Some(index), None) => {
                freed_sector.absorb(
                    self.unused.swap_remove(index)
                );
            }
            (None, Some(index)) => {
                freed_sector.absorb(
                    self.unused.swap_remove(index)
                );
            }
            _ => ()
        }
        if freed_sector.end == self.end_sector.start {
            self.end_sector.absorb(freed_sector);
        } else {
            self.unused.push(freed_sector);
        }
    }

    pub fn reallocate(&mut self, free: SectorOffset, new_size: BlockSize) -> Option<SectorOffset> {
        if new_size.0 == 0 {
            println!("Size is 0");
            return None;
        }

        let old_sector = ManagedSector::from(free);

        if free.block_size().block_count() > new_size.block_count() {
            let (new, old) = old_sector.split_left(new_size.block_count() as u32);
            self.deallocate(old);
            Some(SectorOffset::new(new_size, new.start))
        } else if free.block_size().block_count() == new_size.block_count() {
            Some(free)
        } else {
            self.reallocate_unchecked(free, new_size)
        }
    }

    pub fn reallocate_err(&mut self, free: SectorOffset, size: BlockSize) -> Result<SectorOffset> {
        self.reallocate(free, size).ok_or_else(|| Error::AllocationFailure)
    }
    
    fn reallocate_unchecked(&mut self, free: SectorOffset, new_size: BlockSize) -> Option<SectorOffset> {
        let mut left = Option::<usize>::None;
        let mut right = Option::<usize>::None;
        let mut alloc = Option::<usize>::None;
        let mut freed_sector = ManagedSector::from(free);
        
        fn apply_some_cond(opt: &mut Option<usize>, condition: bool, index: usize) -> bool {
            if condition && opt.is_none() {
                *opt = Some(index);
                true
            } else {
                false
            }
        }
        self.unused.iter().cloned()
            .enumerate()
            .find_map(|(index, sector)| {
                if (
                    apply_some_cond(&mut alloc, sector.size() >= new_size.block_count() as u32, index)
                    || apply_some_cond(&mut left, sector.end == freed_sector.start, index)
                    || apply_some_cond(&mut right, sector.start == freed_sector.end, index)
                )
                && alloc.is_some()
                && left.is_some() 
                && right.is_some() {
                    return Some(());
                }
                None
            });
        enum SuccessAction {
            Replace(usize, ManagedSector),
            Remove(usize),
            None,
        }
        alloc.map(|index| {
            let result = self.unused[index];
            if result.size() > new_size.block_count() as u32 {
                let (new, old) = result.split_left(new_size.block_count() as u32);
                (
                    SectorOffset::new(new_size, new.start),
                    SuccessAction::Replace(index, old)
                )
            } else {
                (
                    SectorOffset::new(new_size, result.start),
                    SuccessAction::Remove(index)
                )
            }
        })
        .or_else(|| {
            self.end_sector
                .allocate(new_size)
                .map(|sector| (sector, SuccessAction::None))
        })
        .map(|(sector, action)| {
            match (left, right) {
                (Some(left), Some(right)) => {
                    freed_sector.absorb(
                        self.unused.swap_remove(right.max(left))
                    );
                    freed_sector.absorb(
                        self.unused.swap_remove(left.min(right))
                    );
                }
                (Some(index), None) => {
                    freed_sector.absorb(
                        self.unused.swap_remove(index)
                    );
                }
                (None, Some(index)) => {
                    freed_sector.absorb(
                        self.unused.swap_remove(index)
                    );
                }
                _ => ()
            }
            if freed_sector.end == self.end_sector.start {
                self.end_sector.absorb(freed_sector);
            } else {
                self.unused.push(freed_sector);
            }
            use SuccessAction::{Replace, Remove};
            match action {
                Replace(index, old) => {
                    self.unused[index] = old;
                }
                Remove(index) => {
                    self.unused.swap_remove(index);
                }
                _ => ()
            }
            sector
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ManagedSector {
    start: u32,
    end: u32
}

impl ManagedSector {
    pub const HEADER: ManagedSector = ManagedSector {
        start: 0,
        end: 12288
    };
    pub const TIMESTAMP_TABLE: ManagedSector = ManagedSector {
        start: 0,
        end: 8192
    };
    pub const SECTOR_TABLE: ManagedSector = ManagedSector {
        start: 8192,
        end: 12288
    };

    
    pub const fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end
        }
    }

    
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    
    pub fn not_empty(self) -> bool {
        self.start != self.end
    }

    
    pub fn size(self) -> u32 {
        self.end - self.start
    }

    /// Allocates a [SectorOffset] from this [ManagedSector], reducing
    /// the size in the process. Returns `None` if there isn't enough
    /// space. This will reduce the size to 0 if that's all the space left.
    /// Does not allow allocation beyond 0x1000000 (2.pow(24))
    
    pub fn allocate(&mut self, size: BlockSize) -> Option<SectorOffset> {
        let block_count = size.block_count();
        let new_start = self.start + block_count as u32;
                                //  2.pow(24) + 8033 (max block size)
        if new_start > self.end.min(0x1001f61) {
            return None;
        }
        let start = self.start;
        self.start = new_start;
        Some(SectorOffset::new(size, start))
    }

    
    pub fn absorb(&mut self, other: Self) {
        if self.end != other.start &&
        self.start != other.end {
            panic!("Nonadjacent sectors");
        }
        self.start = self.start.min(other.start);
        self.end = self.end.max(other.end);
    }

    
    pub fn split_left(self, sector_count: u32) -> (Self, Self) {
        if sector_count > self.size() {
            panic!("Sector not large enough to accomodate sector count.");
        }
        let middle = self.start + sector_count;
        (
            ManagedSector::new(self.start, middle),
            ManagedSector::new(middle, self.end)
        )
    }

    
    pub fn file_offset(self) -> u64 {
        self.start as u64 * 4096
    }

    
    pub fn file_size(self) -> u64 {
        self.size() as u64 * 4096
    }

    
    pub fn gap(self, other: Self) -> Option<u32> {
        if self.end < other.start {
            Some(other.start - self.end)
        } else if other.end < self.start {
            Some(self.start - other.end)
        } else {
            None
        }
    }
}

impl PartialOrd for ManagedSector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.start.partial_cmp(&other.start) {
            Some(core::cmp::Ordering::Equal) => self.end.partial_cmp(&other.end),
            ord => ord,
        }
    }
}

impl Ord for ManagedSector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.start.cmp(&other.start) {
            core::cmp::Ordering::Equal => self.end.cmp(&other.end),
            ord => ord,
        }
    }
}

impl From<SectorOffset> for ManagedSector {
    
    fn from(value: SectorOffset) -> Self {
        let start = value.block_offset();
        let size = value.block_size().block_count() as u32;
        let end = start + size;
        Self::new(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sector_manager_test() {
        let mut man = SectorManager::new();
        let sect1 = man.allocate(BlockSize::MAX_BLOCK_COUNT);
        fn rand_size() -> u16 {
            rand::random::<u16>().rem_euclid(BlockSize::MAX_BLOCK_COUNT).max(1)
        }
        println!("{sect1:?}");
        println!("===First Allocation");
        println!("{:?}", man.unused);
        println!("{:?}", man.end_sector);
        man.deallocate(sect1);
        println!("{:?}", man.unused);
        println!("{:?}", man.end_sector);
        println!("===Allocating Random Sizes");
        let mut sectors: Vec<SectorOffset> = (0..16).map(|_| man.allocate(rand_size())).collect();
        println!("{:?}", man.unused);
        println!("{:?}", man.end_sector);
        println!("===Allocation:");
        println!("{sectors:?}");
        man.deallocate(sectors.remove(7));
        println!("===Removed middle:");
        println!("{:?}", man.unused);
        println!("{:?}", man.end_sector);
        man.deallocate(sectors.remove(7));
        println!("===Removed the next one.");
        println!("{:?}", man.unused);
        println!("{:?}", man.end_sector);
        sectors.into_iter().for_each(|sector| {
            man.deallocate(sector);
        });
        println!("===Deallocated all");
        println!("{:?}", man.unused);
        println!("{:?}", man.end_sector);
    }
}