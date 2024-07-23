use blocks::Id;

use crate::prelude::*;

use super::section::Section;

pub trait Query<'a> {
    type Output;
    fn read(section: &'a Section, index: usize) -> Self::Output;
    fn default() -> Self::Output;
    // fn write(self, section: &mut Section, index: usize) -> Self::Output;
}

// pub trait QueryMut<'a> {
//     type Output;
//     fn read_mut(section: &'a mut Section, index: usize) -> Self::Output;
// }

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockLight(pub u8);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SkyLight(pub u8);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Enabled(pub bool);

impl<'a> Query<'a> for Id {
    type Output = Id;
    
    fn read(section: &'a Section, index: usize) -> Self::Output {
        if let Some(blocks) = &section.blocks {
            blocks[index]
        } else {
            Id::AIR
        }
    }
    
    fn default() -> Self::Output {
        Id::AIR
    }
}

impl<'a> Query<'a> for Occlusion {
    type Output = Occlusion;
    
    fn read(section: &'a Section, index: usize) -> Self::Output {
        if let Some(occlusion) = &section.occlusion {
            occlusion[index]
        } else {
            Occlusion::UNOCCLUDED
        }
    }
    
    fn default() -> Self::Output {
        Occlusion::UNOCCLUDED
    }
}

impl<'a> Query<'a> for BlockLight {
    type Output = u8;
    
    fn read(section: &'a Section, index: usize) -> Self::Output {
        if let Some(light) = &section.block_light {
            // The block light is stored in an array of 2048 bytes, although it's 4096 values.
            // That's 2 values per byte.
            // Using some simple math, we can unpack the value.
            let mask_index = index / 2;
            let sub_index = (index & 1) * 4;
            light[mask_index] >> sub_index & 0xF
        } else {
            0
        }
    }
    
    fn default() -> Self::Output {
        0
    }
}

impl<'a> Query<'a> for SkyLight {
    type Output = u8;
    
    fn read(section: &'a Section, index: usize) -> Self::Output {
        if let Some(light) = &section.sky_light {
            let mask_index = index / 2;
            let sub_index = (index & 1) * 4;
            light[mask_index] >> sub_index & 0xF
        } else {
            15
        }
    }
    
    fn default() -> Self::Output {
        15
    }
}

impl<'a> Query<'a> for Tag {
    type Output = Option<&'a Tag>;
    
    fn read(section: &'a Section, index: usize) -> Self::Output {
        if let Some(data) = &section.block_data_refs {
            let dataref = data[index];
            section.block_data.get(dataref)
        } else {
            None
        }
    }
    
    fn default() -> Self::Output {
        None
    }
}

impl<'a> Query<'a> for Enabled {
    type Output = bool;
    
    fn read(section: &'a Section, index: usize) -> Self::Output {
        if let Some(refs) = &section.update_refs {
            refs[index].enabled()
        } else {
            false
        }
    }
    
    fn default() -> Self::Output {
        false
    }
}

// T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
macro_rules! tuple_query {
    ($($terms:ident),+) => {
        impl<'a, $($terms: Query<'a>,)+> Query<'a> for ($($terms,)+) {
            type Output = ($($terms::Output,)+);
            
            fn read(section: &'a Section, index: usize) -> Self::Output {
                (
                    $(
                        $terms::read(section, index),
                    )+
                )
            }
            
            fn default() -> Self::Output {
                (
                    $(
                        $terms::default(),
                    )+
                )
            }
        }
    };
}

// As of writing this, I only have
// 6 queryable types, but I might add more so I've made it possible for up to 16.
tuple_query!(T0, T1);
tuple_query!(T0, T1, T2);
tuple_query!(T0, T1, T2, T3);
tuple_query!(T0, T1, T2, T3, T4);
tuple_query!(T0, T1, T2, T3, T4, T5);
tuple_query!(T0, T1, T2, T3, T4, T5, T6);
tuple_query!(T0, T1, T2, T3, T4, T5, T6, T7);
tuple_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
tuple_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
tuple_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
tuple_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
tuple_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
tuple_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
tuple_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
tuple_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);

pub trait Mutate {

}