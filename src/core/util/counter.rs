use crate::prelude::*;
use std::sync::atomic::AtomicUsize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Counter(usize);

impl Counter {
    /// Increments the counter and returns the old value.
    #[inline(always)]
    pub fn increment(&mut self) -> usize {
        let next = self.0;
        self.0.swap(self.0 + 1)
    }

    #[inline(always)]
    pub const fn count(self) -> usize {
        self.0
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.0 = 0;
    }
}