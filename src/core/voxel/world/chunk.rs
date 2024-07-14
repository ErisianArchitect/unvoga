use bevy::{asset::Assets, prelude::{state_changed, ResMut}, render::mesh::Mesh, utils::tracing::Instrument};

use crate::core::voxel::{blocks::StateRef, coord::Coord, direction::Direction, rendering::voxelmaterial::VoxelMaterial, tag::Tag};

use super::{dirty::Dirty, heightmap::Heightmap, WORLD_HEIGHT};

fn make_empty_section_blocks() -> Box<[StateRef]> {
    (0..4096).map(|_| StateRef::AIR).collect()
}

fn make_empty_section_light() -> Box<[u8]> {
    (0..2048).map(|_| 0).collect()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockDataRef(u16);

impl BlockDataRef {
    pub const NULL: BlockDataRef = BlockDataRef(0);
    #[inline(always)]
    pub const fn null(self) -> bool {
        self.0 == 0
    }
}

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

    pub fn delete(&mut self, dataref: BlockDataRef) -> Tag {
        if dataref.null() {
            return Tag::Null;
        }
        let index = dataref.0 - 1;
        let tag = self.data[index as usize].take().expect("You done goofed");
        self.unused.push(index);
        if self.unused.len() == self.data.len() {
            self.unused.clear();
            self.unused.shrink_to_fit();
            self.data.clear();
            self.data.shrink_to_fit();
        }
        tag
    }

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

    pub fn dynamic_usage(&self) -> usize {
        let data_size = self.data.capacity() * std::mem::size_of::<Option<Tag>>();
        let unused_size = self.unused.capacity() * 2;
        data_size + unused_size
    }
}

// 4096*4+4096+2048+2048+4096*2
// 32768 bytes
pub struct Section {
    pub blocks: Option<Box<[StateRef]>>,
    pub occlusion: Option<Box<[Occlusion]>>,
    pub block_light: Option<Box<[u8]>>,
    pub sky_light: Option<Box<[u8]>>,
    pub block_data_refs: Option<Box<[BlockDataRef]>>,
    pub block_data: BlockDataContainer,
    pub block_count: u16,
    pub occlusion_count: u16,
    pub block_light_count: u16,
    pub sky_light_count: u16,
    pub block_data_count: u16,
    pub blocks_dirty: Dirty,
    pub light_dirty: Dirty,
    pub section_dirty: Dirty,
}

impl Section {
    pub fn new() -> Self {
        Self {
            blocks: None,
            occlusion: None,
            block_light: None,
            sky_light: None,
            block_data_refs: None,
            block_data: BlockDataContainer::new(),
            block_count: 0,
            occlusion_count: 0,
            block_light_count: 0,
            sky_light_count: 0,
            block_data_count: 0,
            blocks_dirty: Dirty::new(),
            light_dirty: Dirty::new(),
            section_dirty: Dirty::new(),
        }
    }

    pub fn dynamic_usage(&self) -> usize {
        let mut size = 0;
        // let mut printed = false;
        if self.blocks.is_some() {
            // println!("################");
            // printed = true;
            // println!("Blocks Used");
            size += 4096 * std::mem::size_of::<StateRef>();
        }
        if self.occlusion.is_some() {
            // if !printed {
            //     println!("################");
            //     printed = true;
            // }
            // println!("Occlusion Used");
            // println!("{:?}", &self.occlusion);
            size += 4096 * std::mem::size_of::<Occlusion>();
        }
        if self.block_light.is_some() {
            // if !printed {
            //     println!("################");
            //     printed = true;
            // }
            // println!("Block Light Used");
            size += 2048;
        }
        if self.sky_light.is_some() {
            // if !printed {
            //     println!("################");
            //     printed = true;
            // }
            // println!("Sky Light Used");
            size += 2048;
        }
        if self.block_data_refs.is_some() {
            // if !printed {
            //     println!("################");
            //     printed = true;
            // }
            // println!("Block Data Used");
            size += 4096*std::mem::size_of::<BlockDataRef>();
        }
        size + self.block_data.dynamic_usage()
    }

    fn index(coord: Coord) -> usize {
        let x = (coord.x & 0xF) as usize;
        let y = (coord.y & 0xF) as usize;
        let z = (coord.z & 0xF) as usize;
        x | z << 4 | y << 8
    }

    pub fn get(&self, coord: Coord) -> StateRef {
        if let Some(blocks) = &self.blocks {
            let index = Section::index(coord);
            blocks[index]
        } else {
            StateRef::AIR
        }
    }

    pub fn set(&mut self, coord: Coord, state: StateRef) -> SectionUpdate<StateChange> {
        if self.blocks.is_none() {
            if !state.is_air() {
                self.blocks = Some(make_empty_section_blocks());
            } else {
                return SectionUpdate::new(StateChange::Unchanged);
            }
        }
        let index = Section::index(coord);
        let Some(blocks) = &mut self.blocks else {
            panic!("Should have been valid");
        };
        let mut old = state;
        std::mem::swap(&mut blocks[index], &mut old);
        if state != old {
            if !state.is_air() && old.is_air() {
                self.block_count += 1;
            } else if state.is_air() && !old.is_air() {
                self.block_count -= 1;
                if self.block_count == 0 {
                    self.blocks = None;
                }
            }
            self.blocks_dirty.mark();
            if self.section_dirty.mark() {
                SectionUpdate::new_dirty(StateChange::Changed(old))
            } else {
                SectionUpdate::new(StateChange::Changed(old))
            }
        } else {
            SectionUpdate::new(StateChange::Unchanged)
        }
    }

    pub fn occlusion(&self, coord: Coord) -> Occlusion {
        if let Some(occlusion) = &self.occlusion {
            let index = Section::index(coord);
            occlusion[index]
        } else {
            Occlusion::UNOCCLUDED
        }
    }

    pub fn face_visible(&self, coord: Coord, face: Direction) -> bool {
        if let Some(occlusion) = &self.occlusion {
            let index = Section::index(coord);
            occlusion[index].visible(face)
        } else {
            true
        }
    }

    pub fn show_face(&mut self, coord: Coord, face: Direction) -> SectionUpdate<bool> {
        if self.occlusion.is_none() {
            return SectionUpdate::new(true);
        }
        let Some(occlusion) = &mut self.occlusion else {
            unreachable!()
        };
        let index = Section::index(coord);
        let old_flags = occlusion[index];
        let old = occlusion[index].show(face);
        let new_flags = occlusion[index];
        if new_flags == Occlusion::UNOCCLUDED && old_flags != Occlusion::UNOCCLUDED {
            self.occlusion_count -= 1;
            if self.occlusion_count == 0 {
                self.occlusion = None;
            }
        }
        if !old {
            self.blocks_dirty.mark();
            if self.section_dirty.mark() {
                return SectionUpdate::new_dirty(old);
            }
        }
        SectionUpdate::new(old)
    }

    pub fn hide_face(&mut self, coord: Coord, face: Direction) -> SectionUpdate<bool> {
        if self.occlusion.is_none() {
            self.occlusion = Some((0..4096).map(|_| Occlusion::UNOCCLUDED).collect());
        }
        let Some(occlusion) = &mut self.occlusion else {
            unreachable!()
        };
        let index = Section::index(coord);
        let old_flags = occlusion[index];
        let old = occlusion[index].hide(face);
        if old_flags == Occlusion::UNOCCLUDED {
            self.occlusion_count += 1;
        }
        if old {
            self.blocks_dirty.mark();
            if self.section_dirty.mark() {
                return SectionUpdate::new_dirty(old);
            }
        }
        SectionUpdate::new(old)
    }

    /// Returns the maximum of the block light and sky light.
    pub fn get_light(&self, coord: Coord) -> u8 {
        let block_light = self.get_block_light(coord);
        let sky_light = self.get_sky_light(coord);
        block_light.max(sky_light)
    }

    pub fn get_block_light(&self, coord: Coord) -> u8 {
        if let Some(block_light) = &self.block_light {
            let index = Section::index(coord);
            let mask_index = index / 2;
            let sub_index = (index & 1) * 4;
            (block_light[mask_index] & (0xF << sub_index)) >> sub_index
        } else {
            0
        }
    }

    pub fn set_block_light(&mut self, coord: Coord, level: u8) -> SectionUpdate<LightChange> {
        let level = level.min(15);
        let index = Section::index(coord);
        let mask_index = index / 2;
        let sub_index = (index & 1);
        let shift = sub_index * 4;
        // Get the sky light to compare for the max/old max
        let sky_light = if let Some(sky_light) = &self.sky_light {
            (sky_light[mask_index] & (0xF << shift)) >> shift
        } else {
            0
        };
        if self.block_light.is_none() {
            if level != 0 {
                self.block_light = Some(make_empty_section_light());
            } else {
                return SectionUpdate::new(LightChange {
                    old_max: sky_light,
                    old_level: 0,
                    new_level: 0,
                    new_max: sky_light,
                });
            }
        }
        let Some(block_light) = &mut self.block_light else {
            panic!("Should be valid");
        };
        let other_index = ((sub_index as i32 - 1) & 1) as usize;
        let other_shift = other_index * 4;
        let old_level = (block_light[mask_index] & (0xF << shift)) >> shift;
        let old_max = sky_light.max(old_level);
        let change = LightChange {
            old_max,
            old_level,
            new_level: level,
            new_max: sky_light.max(level),
        };
        if level == old_level {
            return SectionUpdate::new(change);
        }
        let other = block_light[mask_index] & (0xF << other_shift);
        block_light[mask_index] = other | (level << shift);

        if level != 0 && old_level == 0 {
            self.block_light_count += 1;
        } else if level == 0 && old_level != 0 {
            self.block_light_count -= 1;
            if self.block_light_count == 0 {
                self.block_light = None;
            }
        }
        if change.new_max != change.old_max {
            self.light_dirty.mark();
            if self.section_dirty.mark() {
                SectionUpdate::new_dirty(change)
            } else {
                SectionUpdate::new(change)
            }
        } else {
            SectionUpdate::new(change)
        }
    }

    pub fn get_sky_light(&self, coord: Coord) -> u8 {
        if let Some(sky_light) = &self.sky_light {
            let index = Section::index(coord);
            let mask_index = index / 2;
            let sub_index = (index & 1) * 4;
            (sky_light[mask_index] & (0xF << sub_index)) >> sub_index
        } else {
            0
        }
    }

    pub fn set_sky_light(&mut self, coord: Coord, level: u8) -> SectionUpdate<LightChange> {
        let level = level.min(15);
        let index = Section::index(coord);
        let mask_index = index / 2;
        let sub_index = (index & 1);
        let shift = sub_index * 4;
        // Get the block light to compare for the max/old max
        let block_light = if let Some(block_light) = &self.block_light {
            (block_light[mask_index] & (0xF << shift)) >> shift
        } else {
            0
        };
        if self.sky_light.is_none() {
            if level != 0 {
                self.sky_light = Some(make_empty_section_light());
            } else {
                return SectionUpdate::new(LightChange {
                    old_max: block_light,
                    old_level: 0,
                    new_level: 0,
                    new_max: block_light,
                });
            }
        }
        let Some(sky_light) = &mut self.sky_light else {
            panic!("Should be valid");
        };
        // light values are packed as 4-bit nibbles
        // gotta unpack the other value
        let other_index = ((sub_index as i32 - 1) & 1) as usize;
        let other_shift = other_index * 4;
        let old_level = (sky_light[mask_index] & (0xF << shift)) >> shift;
        let old_max = block_light.max(old_level);
        let change = LightChange {
            old_max,
            old_level,
            new_level: level,
            new_max: block_light.max(level),
        };
        if level == old_level {
            return SectionUpdate::new(change);
        }
        let other = sky_light[mask_index] & (0xF << other_shift);
        sky_light[mask_index] = other | (level << shift);
        if level != 0 && old_level == 0 {
            self.sky_light_count += 1;
        } else if level == 0 && old_level != 0 {
            self.sky_light_count -= 1;
            if self.sky_light_count == 0 {
                self.sky_light = None;
            }
        }
        if change.new_max != change.old_max {
            self.light_dirty.mark();
            if self.section_dirty.mark() {
                SectionUpdate::new_dirty(change)
            } else {
                SectionUpdate::new(change)
            }
        } else {
            SectionUpdate::new(change)
        }
    }

    pub fn get_data(&self, coord: Coord) -> Option<&Tag> {
        let Some(data) = &self.block_data_refs else {
            return None;
        };
        let index = Section::index(coord);
        let dref = data[index];
        self.block_data.get(dref)
    }

    pub fn get_data_mut(&mut self, coord: Coord) -> Option<&mut Tag> {
        let Some(data) = &mut self.block_data_refs else {
            return None;
        };
        let index = Section::index(coord);
        let dref = data[index];
        self.block_data.get_mut(dref)
    }

    pub fn delete_data(&mut self, coord: Coord) -> Option<Tag> {
        let Some(data) = &mut self.block_data_refs else {
            return None;
        };
        let index = Section::index(coord);
        let mut old = BlockDataRef::NULL;
        std::mem::swap(&mut old, &mut data[index]);
        if !old.null() {
            self.block_data_count -= 1;
            if self.block_data_count == 0 {
                self.block_data_refs = None;
            }
            Some(self.block_data.delete(old))
        } else {
            None
        }
    }

    pub fn set_data(&mut self, coord: Coord, tag: Tag) -> Option<Tag> {
        let index = Section::index(coord);
        let (oldref, data) = if let Some(data) = &mut self.block_data_refs {
            (data[index], data)
        } else {
            let data = self.block_data_refs.insert((0..4096).map(|_| BlockDataRef::NULL).collect());
            (BlockDataRef::NULL, data)
        };
        let old = if oldref.null() {
            self.block_data_count += 1;
            None
        } else {
            Some(self.block_data.delete(oldref))
        };
        let new = self.block_data.insert(tag);
        data[index] = new;
        old
    }

    /// Copy max block/sky light into dest (where dest is 4096 slot lightmap stored yzx order).
    pub fn copy_lightmap(&self, dest: &mut [u8]) {
        (0..4096).for_each(|i| {
            let mask_index = i / 2;
            let sub_index = (i & 1);
            let shift = sub_index * 4;
            let block_light = if let Some(block_light) = &self.block_light {
                (block_light[mask_index] & (0xF << shift)) >> shift
            } else {
                0
            };
            let sky_light = if let Some(sky_light) = &self.sky_light {
                (sky_light[mask_index] & (0xF << shift)) >> shift
            } else {
                0
            };
            let light = block_light.max(sky_light);
            dest[i] = light;
        });
    }
}

pub struct Chunk {
    pub sections: Box<[Section]>,
    pub heightmap: Heightmap,
    /// The offset block coordinate.
    pub block_offset: Coord,
}

impl Chunk {
    const SECTION_COUNT: usize = WORLD_HEIGHT / 16;
    pub fn new(offset: Coord) -> Self {
        Self {
            sections: (0..Self::SECTION_COUNT).map(|_| Section::new()).collect(),
            heightmap: Heightmap::new(),
            block_offset: offset,
        }
    }

    pub fn dynamic_usage(&self) -> usize {
        self.sections.iter().map(|section| section.dynamic_usage()).sum()
    }

    pub fn get(&self, coord: Coord) -> StateRef {
        // no bounds check because this will only be called by
        // the world, which will already be bounds checked.
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get(coord)
    }

    pub fn set(&mut self, coord: Coord, value: StateRef) -> SectionUpdate<StateChange> {
        // no bounds check because this will only be called by
        // the world, which will already be bounds checked.
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        let nonair = value != StateRef::AIR;
        self.heightmap.set(Coord::new(coord.x, coord.y - self.block_offset.y, coord.z), nonair);
        self.sections[section_index].set(coord, value)
    }

    pub fn occlusion(&self, coord: Coord) -> Occlusion {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].occlusion(coord)
    }

    pub fn face_visible(&self, coord: Coord, face: Direction) -> bool {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].face_visible(coord, face)
    }

    pub fn show_face(&mut self, coord: Coord, face: Direction) -> SectionUpdate<bool> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].show_face(coord, face)
    }

    pub fn hide_face(&mut self, coord: Coord, face: Direction) -> SectionUpdate<bool> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].hide_face(coord, face)
    }
    
    /// Returns the maximum of the block light and sky light.
    pub fn get_light(&self, coord: Coord) -> u8 {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_light(coord)
    }

    pub fn get_block_light(&self, coord: Coord) -> u8 {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_block_light(coord)
    }

    pub fn get_sky_light(&self, coord: Coord) -> u8 {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_sky_light(coord)
    }

    pub fn set_block_light(&mut self, coord: Coord, level: u8) -> SectionUpdate<LightChange> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].set_block_light(coord, level)
    }

    pub fn set_sky_light(&mut self, coord: Coord, level: u8) -> SectionUpdate<LightChange> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].set_sky_light(coord, level)
    }

    pub fn get_data(&self, coord: Coord) -> Option<&Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_data(coord)
    }

    pub fn get_data_mut(&mut self, coord: Coord) -> Option<&mut Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].get_data_mut(coord)
    }

    pub fn delete_data(&mut self, coord: Coord) -> Option<Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].delete_data(coord)
    }

    pub fn set_data(&mut self, coord: Coord, tag: Tag) -> Option<Tag> {
        let section_index = (coord.y - self.block_offset.y) as usize / 16;
        self.sections[section_index].set_data(coord, tag)
    }

    pub fn height(&self, x: i32, z: i32) -> i32 {
        self.heightmap.height(x, z) + self.block_offset.y
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LightChange {
    pub old_max: u8,
    pub old_level: u8,
    pub new_max: u8,
    pub new_level: u8,
}

impl LightChange {
    #[inline]
    pub fn changed(self) -> bool {
        self.new_level != self.old_level
    }

    #[inline]
    pub fn max_changed(self) -> bool {
        self.new_max != self.old_max
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateChange {
    Unchanged,
    Changed(StateRef)
}

impl StateChange {
    pub fn changed(self) -> bool {
        matches!(self, StateChange::Changed(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionUpdate<T: Clone + Copy + PartialEq + Eq + std::hash::Hash> {
    /// Determines if the section was marked dirty during this update.
    pub marked_dirty: bool,
    pub change: T,
}

impl<T: Clone + Copy + PartialEq + Eq + std::hash::Hash> SectionUpdate<T> {
    pub fn new_dirty(change: T) -> Self {
        Self {
            marked_dirty: true,
            change
        }
    }

    /// Not marked dirty
    pub fn new(change: T) -> Self {
        Self {
            marked_dirty: false,
            change
        }
    }
}


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Occlusion(u8);

macro_rules! make_face_constants {
    ($($name:ident = $dir:ident;)*) => {
        $(
            pub const $name: Self = Occlusion(1 << Direction::$dir as u8);
        )*
    };
}

impl Occlusion {
    pub const UNOCCLUDED: Self = Occlusion(0);
    pub const OCCLUDED: Self = Occlusion(0b111111);
    make_face_constants!(
        NEG_X = NegX;
        NEG_Y = NegY;
        NEG_Z = NegZ;
        POS_X = PosX;
        POS_Y = PosY;
        POS_Z = PosZ;
    );
    const FLAGS_MASK: u8 = 0b111111;

    #[inline]
    pub fn fully_occluded(self) -> bool {
        self == Self::OCCLUDED
    }

    #[inline]
    pub fn show(&mut self, face: Direction) -> bool {
        let bit = face.bit();
        let old = self.0 & bit == bit;
        self.0 = self.0 & !bit;
        old
    }

    #[inline]
    pub fn hide(&mut self, face: Direction) -> bool {
        let bit = face.bit();
        let old = self.0 & bit == bit;
        self.0 = self.0 | bit;
        old
    }

    #[inline]
    pub fn visible(self, face: Direction) -> bool {
        let bit = face.bit();
        self.0 & bit != bit
    }

    #[inline]
    pub fn hidden(self, face: Direction) -> bool {
        let bit = face.bit();
        self.0 & bit == bit
    }

    #[inline]
    pub fn neg_x(self) -> bool {
        self.visible(Direction::NegX)
    }

    #[inline]
    pub fn neg_y(self) -> bool {
        self.visible(Direction::NegY)
    }

    #[inline]
    pub fn neg_z(self) -> bool {
        self.visible(Direction::NegZ)
    }

    #[inline]
    pub fn pos_x(self) -> bool {
        self.visible(Direction::PosX)
    }

    #[inline]
    pub fn pos_y(self) -> bool {
        self.visible(Direction::PosY)
    }

    #[inline]
    pub fn pos_z(self) -> bool {
        self.visible(Direction::PosZ)
    }
}

impl std::ops::BitOr<Occlusion> for Occlusion {
    type Output = Occlusion;
    #[inline]
    fn bitor(self, rhs: Occlusion) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd<Occlusion> for Occlusion {
    type Output = Occlusion;
    #[inline]
    fn bitand(self, rhs: Occlusion) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::Sub<Occlusion> for Occlusion {
    type Output = Occlusion;
    #[inline]
    fn sub(self, rhs: Occlusion) -> Self::Output {
        Self(self.0 & !rhs.0)
    }
}

impl std::ops::BitAnd<Direction> for Occlusion {
    type Output = bool;
    fn bitand(self, rhs: Direction) -> Self::Output {
        self.visible(rhs)
    }
}

impl std::fmt::Display for Occlusion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Occlusion(")?;
        Direction::iter().try_fold(false, |mut sep, dir| {
            if self.hidden(dir) {
                if sep {
                    write!(f, "|")?;
                }
                sep = true;
                write!(f, "{dir:?}")?;
            }
            Ok(sep)
        })?;
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use crate::core::voxel::{direction::Direction, world::chunk::Occlusion};

    #[test]
    fn occlusion_test() {
        let mut occlusion = Occlusion::UNOCCLUDED;
        Direction::iter().for_each(|dir| {
            assert!(!occlusion.hidden(dir));
            assert!(occlusion.visible(dir));
            occlusion.hide(dir);
            assert!(occlusion.hidden(dir));
            assert!(!occlusion.visible(dir));
            occlusion.show(dir);
            assert!(!occlusion.hidden(dir));
            assert!(occlusion.visible(dir));
        });
        assert_eq!(occlusion, Occlusion::UNOCCLUDED);
        assert!(Occlusion::OCCLUDED.fully_occluded());
    }
    
}
