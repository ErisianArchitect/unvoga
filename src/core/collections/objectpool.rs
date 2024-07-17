use std::{num::NonZeroU64, sync::atomic::AtomicU64};


/// An unordered object pool with O(1) lookup, insertion, deletion, and iteration.
/// Sounds too good to be true!
#[derive(Debug)]
pub struct ObjectPool<T> {
    pool: Vec<(T, PoolId)>,
    indices: Vec<usize>,
    unused: Vec<PoolId>,
    id: u64,
}

impl<T> ObjectPool<T> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            pool: Vec::new(),
            indices: Vec::new(),
            unused: Vec::new(),
            id: Self::next_id(),
        }
    }

    #[must_use]
    #[inline(always)]
    fn next_id() -> u64 {
        static mut ID: AtomicU64 = AtomicU64::new(0);
        unsafe {
            ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[must_use]
    pub fn insert(&mut self, value: T) -> PoolId {
        if let Some(unused_index) = self.unused.pop() {
            let new_id = unused_index.inc_gen();
            let pool_index = self.indices[new_id.index()];
            self.pool[pool_index] = (value, new_id);
            new_id
        } else {
            let index = self.indices.len();
            let pool_index = self.pool.len();
            let id = PoolId::new(self.id, index, 0);
            self.pool.push((value, id));
            self.indices.push(pool_index);
            id
        }
    }

    pub fn remove(&mut self, id: PoolId) {
        if id.null() {
            return;
        }
        if id.pool_id() != self.id {
            panic!("Id does not belong to this pool.");
        }
        let pool_index = self.indices[id.index()];
        if self.pool[pool_index].1 != id {
            panic!("Dead pool ID");
        }
        self.pool.swap_remove(pool_index);
        if pool_index == self.pool.len() {
            return;
        }
        let index_index = self.pool[pool_index].1;
        self.indices[index_index.index()] = pool_index;
        self.unused.push(id);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pool: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(capacity),
            unused: Vec::new(),
            id: Self::next_id(),
        }
    }

    #[inline]
    #[must_use]
    pub fn get(&self, id: PoolId) -> Option<&T> {
        if id.null() || id.pool_id() != self.id {
            return None;
        }
        let pool_index = self.indices[id.index()];
        if self.pool[pool_index].1 != id {
            return None;
        }
        Some(&self.pool[pool_index].0)
    }

    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, id: PoolId) -> Option<&mut T> {
        if id.null() || id.pool_id() != self.id {
            return None;
        }
        let pool_index = self.indices[id.index()];
        if self.pool[pool_index].1 != id {
            return None;
        }
        Some(&mut self.pool[pool_index].0)
    }

    #[inline(always)]
    #[must_use]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.pool.iter().map(|(item, _)| item)
    }

    #[inline(always)]
    #[must_use]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.pool.iter_mut().map(|(item,_)| item)
    }
}

impl<T> std::ops::Index<PoolId> for ObjectPool<T> {
    type Output = T;
    fn index(&self, index: PoolId) -> &Self::Output {
        self.get(index).expect("PoolId was invalid")
    }
}

impl<T> std::ops::IndexMut<PoolId> for ObjectPool<T> {
    fn index_mut(&mut self, index: PoolId) -> &mut Self::Output {
        self.get_mut(index).expect("PoolId was invalid")
    }
}

impl std::fmt::Display for PoolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PoolId(pool_id={},index={},generation={})", self.pool_id(), self.index(), self.generation())
    }
}

// TODO: Maybe consider using PhantomData marker to distinguish PoolId subtypes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PoolId(u64);

impl PoolId {
    const           INDEX_BITS: u64 = 0b0000000000000000000000000000000000000011111111111111111111111111;
    const         POOL_ID_BITS: u64 = 0b0000000000000000000011111111111111111100000000000000000000000000;
    const      GENERATION_BITS: u64 = 0b1111111111111111111100000000000000000000000000000000000000000000;
    const       GENERATION_MAX: u64 = 0b0000000000000000000000000000000000000000000011111111111111111111; 
    const          POOL_ID_MAX: u64 = 0b0000000000000000000000000000000000000000000000111111111111111111;
    // For readability I guess.
    const            INDEX_MAX: u64 = Self::INDEX_BITS;
    const       POOL_ID_OFFSET: u32 = 26;
    const GENERATION_ID_OFFSET: u32 = 44;
    pub const NULL: Self = Self(0);

    #[inline(always)]
    #[must_use]
    pub fn null(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    #[must_use]
    fn new(pool_id: u64, index: usize, generation: u64) -> Self {
        let index = index as u64 + 1;
        if index > Self::INDEX_BITS {
            panic!("Index out of bounds.");
        }
        if generation > Self::GENERATION_MAX {
            panic!("Generation out of bounds.");
        }
        if pool_id > Self::POOL_ID_MAX {
            panic!("Pool Id out of bounds.");
        }
        Self(index | pool_id << Self::POOL_ID_OFFSET | generation << Self::GENERATION_ID_OFFSET)
    }

    #[inline(always)]
    #[must_use]
    pub fn id(self) -> u64 {
        self.0
    }

    #[inline(always)]
    #[must_use]
    pub fn index(self) -> usize {
        ((self.0 & Self::INDEX_BITS) as usize) - 1
    }

    #[inline(always)]
    #[must_use]
    pub fn generation(self) -> u64 {
        self.0 >> Self::GENERATION_ID_OFFSET
    }

    #[inline(always)]
    #[must_use]
    pub fn pool_id(self) -> u64 {
        self.0 >> Self::POOL_ID_OFFSET & Self::POOL_ID_MAX
    }

    /// Increment Generation
    #[inline(always)]
    #[must_use]
    fn inc_gen(self) -> Self {
        let pool_id = self.pool_id();
        let index = self.index();
        let generation = self.generation();
        Self::new(pool_id, index, generation + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pool_test() {
        let mut pool = ObjectPool::new();
        let hello = pool.insert("Hello, world!");
        let test = pool.insert("Test string");
        let bob = pool.insert("Bob");
        println!("{}", pool[hello]);
        pool.remove(hello);
        let fox = pool.insert("The quick brown fox jumps over the lazy dog.");
        println!("{} -> {}", hello, fox);
        println!("{}", pool.get(hello).is_none());
        pool.iter().for_each(|s| println!("{}", s));

    }
}