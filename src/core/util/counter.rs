use crate::prelude::*;
use std::{sync::atomic::{AtomicU64, AtomicUsize, Ordering}, time::Duration};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Counter(u64);

impl Counter {
    /// Increments the counter and returns the old value.
    #[inline(always)]
    pub fn increment(&mut self) -> u64 {
        let next = self.0;
        self.0.swap(self.0 + 1)
    }

    #[inline(always)]
    pub const fn count(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.0 = 0;
    }
}

#[derive(Debug, Default)]
pub struct AtomicCounter(AtomicU64);

impl AtomicCounter {
    /// Increments the counter and returns the old value.
    #[inline(always)]
    pub fn increment(&mut self) -> u64 {
        self.0.fetch_add(1, Ordering::SeqCst)
    }

    #[inline(always)]
    pub fn count(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.0.store(0, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::{Arc, OnceLock}, thread};

    use super::blocks::Id;
    #[test]
    fn sandbox() {
        
    }
    
}