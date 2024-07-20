use crate::core::voxel::coord::Coord;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UpdateRef(u32);

impl UpdateRef {
    pub const NULL: Self = Self(0);

    
    pub const fn null(self) -> bool {
        self.0 == 0
    }

    
    pub const fn enabled(self) -> bool {
        self.0 != 0
    }

    
    pub const fn disabled(self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Default)]
pub struct BlockUpdateQueue {
    pub update_queue: Vec<(Coord, u32)>,
    ref_key: Vec<Option<u32>>,
    unused_refs: Vec<u32>,
}

impl BlockUpdateQueue {
    
    pub fn push(&mut self, coord: Coord) -> UpdateRef {
        let ref_index = if let Some(ref_index) = self.unused_refs.pop() {
            ref_index
        } else {
            let index = self.ref_key.len() as u32;
            self.ref_key.push(None);
            index
        };
        let update_index = self.update_queue.len() as u32;
        self.update_queue.push((coord, ref_index));
        self.ref_key[ref_index as usize] = Some(update_index);
        UpdateRef(ref_index + 1)
    }

    
    pub fn remove(&mut self, key: UpdateRef) {
        if key.null() {
            return;
        }
        let Some(ref_index) = self.ref_key[key.0 as usize - 1] else {
            panic!("Ref Key pointed to None");
        };
        let ref_index = ref_index as usize;
        self.update_queue.swap_remove(ref_index);
        self.unused_refs.push(key.0 - 1);
        if ref_index == self.update_queue.len() {
            return;
        }
        let fix_index = self.update_queue[ref_index].1;
        let Some(fix) = &mut self.ref_key[fix_index as usize] else {
            panic!("fix was None");
        };
        *fix = ref_index as u32;
    }

    
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a Coord> {
        self.update_queue.iter().map(|(coord, _)| coord)
    }
}

#[test]
fn quick() {
    let mut update = BlockUpdateQueue::default();
    let mut keys = Vec::new();
    for i in 0..16 {
        keys.push(update.push(Coord::new(i,i,i)));
    }
    update.iter().cloned().for_each(|coord| {
        println!("{coord}");
    });
    keys.into_iter().for_each(|key| {
        update.remove(key);
    });
    update.iter().cloned().for_each(|coord| {
        println!("{coord}");
    });
}