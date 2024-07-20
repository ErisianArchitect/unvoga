use std::any::Any;

use crate::core::voxel::tag::Tag;

use super::MemoryUsage;

// pub trait BlockData: Any {
//     fn as_any(&self) -> &dyn Any;
//     fn as_any_mut(&mut self) -> &mut dyn Any;
// }

/// Block data is store in a chunk section as a [Tag]. There are up to 4096 slots
/// available for block data in a single section (one [Tag] per block).
/// This block data is stored as a [u16] index that points to the [Tag] in
/// the [BlockDataContainer].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockDataRef(u16);

impl BlockDataRef {
    pub const NULL: BlockDataRef = BlockDataRef(0);
    /// Checks to see if this is the NULL value (0).
    
    pub const fn null(self) -> bool {
        self.0 == 0
    }
}

/// A container for block data. There are up to 4096 slots (one [Tag] per block in a [Section])
pub struct BlockDataContainer {
    data: Vec<Option<Tag>>,
    unused: Vec<u16>,
}

impl BlockDataContainer {
    
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            unused: Vec::new(),
        }
    }

    /// Insert new data returning the reference to the slot.
    
    pub fn insert<T: Into<Tag>>(&mut self, tag: T) -> BlockDataRef {
        if let Some(index) = self.unused.pop() {
            self.data[index as usize] = Some(tag.into());
            BlockDataRef(index + 1)
        } else {
            if self.data.len() == 4096 {
                panic!("BlockDataContainer overflow. This should contain at most 4096 items.")
            }
            let index = self.data.len() as u16;
            self.data.push(Some(tag.into()));
            BlockDataRef(index + 1)
        }
    }

    /// Delete data from the container, returning the data that was deleted.
    
    pub fn delete(&mut self, dataref: BlockDataRef) -> Tag {
        if dataref.null() {
            return Tag::Null;
        }
        let index = dataref.0 - 1;
        let tag = self.data[index as usize].take().expect("You done goofed");
        self.unused.push(index);
        if self.unused.len() == self.data.len() {
            self.clear();
        }
        tag
    }

    /// Get a reference to the [Tag] data.
    
    pub fn get(&self, dataref: BlockDataRef) -> Option<&Tag> {
        if dataref.null() {
            return None;
        }
        let index = dataref.0 - 1;
        let Some(tag) = &self.data[index as usize] else {
            panic!("Data was None, which shouldn't have happened.");
        };
        Some(tag)
    }

    /// Get a mutable reference to the [Tag] data.
    
    pub fn get_mut(&mut self, dataref: BlockDataRef) -> Option<&mut Tag> {
        if dataref.null() {
            return None;
        }
        let index = dataref.0 - 1;
        let Some(tag) = &mut self.data[index as usize] else {
            panic!("Data was None, which shouldn't have happened.");
        };
        Some(tag)
    }

    /// Gets the dynamic memory usage.
    
    pub fn dynamic_usage(&self) -> MemoryUsage {
        let data_size = self.data.capacity() * std::mem::size_of::<Option<Tag>>();
        let unused_size = self.unused.capacity() * 2;
        MemoryUsage::new(
            data_size + unused_size,
            4096 * std::mem::size_of::<Option<Tag>>() + 4096*2
        )
    }

    
    pub fn clear(&mut self) {
        self.data.clear();
        self.data.shrink_to_fit();
        self.unused.clear();
        self.unused.shrink_to_fit();
    }
}

// #[test]
// pub fn test() {
//     struct Data(&'static str);
//     impl BlockData for Data {
//         fn as_any(&self) -> &dyn Any {
//             self
//         }

//         fn as_any_mut(&mut self) -> &mut dyn Any {
//             self
//         }
//     }
//     let data: Box<dyn BlockData> = Box::new(Data("Test"));
//     let dat: Option<&Data> = data.as_any().downcast_ref();
//     if let Some(dat) = dat {
//         println!("{}", dat.0);
//     }
// }