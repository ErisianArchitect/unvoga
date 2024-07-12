use std::sync::atomic::AtomicBool;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dirty(bool);

impl Dirty {
    pub const CLEAN: Dirty = Dirty(false);
    pub const DIRTY: Dirty = Dirty(true);

    #[inline]
    pub const fn new() -> Self {
        Self(false)
    }
    
    /// Returns false if already marked dirty, otherwise returns true.
    #[inline]
    pub fn mark(&mut self) -> bool {
        let mut old = true;
        std::mem::swap(&mut self.0, &mut old);
        !old
    }

    #[inline]
    pub fn mark_clean(&mut self) {
        self.0 = false;
    }

    #[inline]
    pub fn dirty(self) -> bool {
        self.0
    }
}