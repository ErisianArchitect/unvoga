use std::sync::atomic::AtomicUsize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Counter(usize);

impl Counter {
    /// Increments the counter and returns the old value.
    #[inline(always)]
    pub fn increment(&mut self) -> usize {
        let next = self.0;
        self.0 += 1;
        next
    }

    #[inline(always)]
    pub fn count(self) -> usize {
        self.0
    }
}