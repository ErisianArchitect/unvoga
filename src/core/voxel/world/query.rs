use blocks::StateRef;

use crate::prelude::*;

use super::section::Section;

pub trait Queryable<'a> {
    type Output;
    fn read(section: &'a mut Section, index: usize) -> Self::Output;
    // fn write(self, section: &mut Section, index: usize) -> Self::Output;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockLight(pub u8);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SkyLight(pub u8);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Enabled(pub bool);

impl<'a> Queryable<'a> for StateRef {
    type Output = StateRef;
    fn read(section: &'a mut Section, index: usize) -> Self::Output {
        if let Some(blocks) = &section.blocks {
            blocks[index]
        } else {
            StateRef::AIR
        }
    }
}

impl<'a> Queryable<'a> for Occlusion {
    type Output = Occlusion;
    fn read(section: &'a mut Section, index: usize) -> Self::Output {
        if let Some(occlusion) = &section.occlusion {
            occlusion[index]
        } else {
            Occlusion::UNOCCLUDED
        }
    }
}

impl<'a> Queryable<'a> for BlockLight {
    type Output = u8;
    fn read(section: &'a mut Section, index: usize) -> Self::Output {
        if let Some(light) = &section.block_light {
            light[index]
        } else {
            0
        }
    }
}

impl<'a> Queryable<'a> for SkyLight {
    type Output = u8;
    fn read(section: &'a mut Section, index: usize) -> Self::Output {
        if let Some(light) = &section.sky_light {
            light[index]
        } else {
            0
        }
    }
}

impl<'a> Queryable<'a> for &'a Tag {
    type Output = Option<&'a Tag>;
    fn read(section: &'a mut Section, index: usize) -> Self::Output {
        if let Some(data) = &section.block_data_refs {
            let dataref = data[index];
            section.block_data.get(dataref)
        } else {
            None
        }
    }
}

impl<'a> Queryable<'a> for &'a mut Tag {
    type Output = Option<&'a mut Tag>;
    fn read(section: &'a mut Section, index: usize) -> Self::Output {
        if let Some(data) = &mut section.block_data_refs {
            let dataref = data[index];
            section.block_data.get_mut(dataref)
        } else {
            None
        }
    }
}

impl<'a> Queryable<'a> for Enabled {
    type Output = bool;
    fn read(section: &'a mut Section, index: usize) -> Self::Output {
        if let Some(refs) = &section.update_refs {
            refs[index].enabled()
        } else {
            false
        }
    }
}