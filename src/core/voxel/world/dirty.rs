use std::sync::atomic::AtomicBool;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dirty(bool);

impl Dirty {
    /// Returns false if already marked dirty, otherwise returns true.
    pub fn mark(&mut self) -> bool {
        let mut old = true;
        std::mem::swap(&mut self.0, &mut old);
        !old
    }

    pub fn dirty(self) -> bool {
        self.0
    }
}