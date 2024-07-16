use std::ops::Range;

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

    pub fn reallocate(&mut self, free: SectorOffset, blocks_required: u16) -> SectorOffset {
        if blocks_required == 0 {
            panic!("Requested size of 0.");
        }

        let new_size = BlockSize::required(blocks_required);
        let old_sector = ManagedSector::from(free);

        if free.block_size().block_count() > blocks_required {
            let (new, old) = old_sector.split_left(new_size.block_count() as u32);
            self.deallocate(old);
            SectorOffset::new(new_size, new.start)
        } else if free.block_size().block_count() == new_size.block_count() {
            free
        } else {
            self.reallocate_unchecked(free, blocks_required)
        }
    }

    fn reallocate_unchecked(&mut self, free: SectorOffset, new_size: u16) -> SectorOffset {
        let mut left = Option::<usize>::None;
        let mut right = Option::<usize>::None;
        let mut alloc = Option::<usize>::None;
        let mut freed_sector = ManagedSector::from(free);
        let block_size = BlockSize::required(new_size);
        #[inline]
        fn apply_some_cond(opt: &mut Option<usize>, condition: bool, index: usize) -> bool {
            if opt.is_none() && condition {
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
                    apply_some_cond(&mut alloc, sector.size() >= new_size as u32, index)
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
            if result.size() > new_size as u32 {
                let (new, old) = result.split_left(block_size.block_count() as u32);
                (
                    SectorOffset::new(block_size, new.start),
                    SuccessAction::Replace(index, old)
                )
            } else {
                (
                    SectorOffset::new(block_size, result.start),
                    SuccessAction::Remove(index)
                )
            }
        })
        .or_else(|| {
            self.end_sector
                .allocate(block_size)
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
        }).expect("Failed operation.")
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end
        }
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub fn size(self) -> u32 {
        self.end - self.start
    }

    /// Allocates a [SectorOffset] from this [ManagedSector], reducing
    /// the size in the process. Returns `None` if there isn't enough
    /// space. This will reduce the size to 0 if that's all the space left.
    /// Does not allow allocation beyond 0x1000000 (2.pow(24))
    #[inline]
    pub fn allocate(&mut self, size: BlockSize) -> Option<SectorOffset> {
        let block_count = size.block_count();
        let new_start = self.start + block_count as u32;
        if new_start > self.end.min(0x1001f61) {
            return None;
        }
        let start = self.start;
        self.start = new_start;
        Some(SectorOffset::new(size, start))
    }

    #[inline]
    pub fn absorb(&mut self, other: Self) {
        if self.end != other.start &&
        self.start != other.end {
            panic!("Nonadjacent sectors");
        }
        self.start = self.start.min(other.start);
        self.end = self.end.max(other.end);
    }

    // #[inline]
    // pub fn sector_offset(self) -> u32 {
    //     self.start * 4096
    // }

    // #[inline]
    // pub fn sector_count(self) -> u32 {
    //     self.size() * 4096
    // }
    #[inline]
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
}

impl From<SectorOffset> for ManagedSector {
    #[inline]
    fn from(value: SectorOffset) -> Self {
        let start = value.block_offset();
        let size = value.block_size().block_count() as u32;
        let end = start + size;
        Self::new(start, end)
    }
}