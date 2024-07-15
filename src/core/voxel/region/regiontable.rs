use crate::prelude::*;

pub trait RegionTableItem: Default + Copy {
    const OFFSET: u64;
}

pub struct RegionTable<T: RegionTableItem> {
    table: Box<[T]>,
}

impl<T: RegionTableItem> RegionTable<T> {
    pub fn new() -> Self {
        Self {
            table: (0..1024).map(|_| T::default()).collect(),
        }
    }

    pub fn get(&self, x: i32, y: i32) -> T {
        let index = region_index(x, y);
        self.table[index]
    }

    pub fn set(&mut self, x: i32, y: i32, value: T) -> T {
        let index = region_index(x, y);
        self.table[index].swap(value)
    }
}

#[inline(always)]
pub fn region_index(x: i32, y: i32) -> usize {
    let x = (x & 0b11111) as usize;
    let y = (y & 0b11111) as usize;
    x | y << 5
}