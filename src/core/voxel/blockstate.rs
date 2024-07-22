use std::{borrow::Borrow, fmt::Debug, ops::{Index, IndexMut}};

use bevy::math::{IVec2, IVec3};
use itertools::Itertools;
use crate::core::{error::*, math::coordmap::Orientation};

use crate::{core::{math::coordmap::{Flip, Rotation}, util::traits::StrToOwned}, prelude::{read_u24, write_u24, Readable, Writeable}};

use super::{axis::Axis, blocks::{self, Id}, coord::Coord, direction::{Cardinal, Direction}, world::chunkcoord::ChunkCoord};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockState {
    name: String,
    sorted_properties: Vec<BlockProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockProperty {
    name: String,
    value: StateValue
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum StateValue {
    #[default]
    Null = 0,
    Int(i64) = 1,
    Bool(bool) = 2,
    String(String) = 3,
    Direction(Direction) = 4,
    Cardinal(Cardinal) = 5,
    Rotation(Rotation) = 6,
    Flip(Flip) = 7,
    Orientation(Orientation) = 8,
    Axis(Axis) = 9,
    Coord2(IVec2) = 10,
    Coord3(IVec3) = 11,
}

impl StateValue {
    
    pub fn id(&self) -> u8 {
        match self {
            StateValue::Null => 0,
            StateValue::Int(_) => 1,
            StateValue::Bool(_) => 2,
            StateValue::String(_) => 3,
            StateValue::Direction(_) => 4,
            StateValue::Cardinal(_) => 5,
            StateValue::Rotation(_) => 6,
            StateValue::Flip(_) => 7,
            StateValue::Orientation(_) => 8,
            StateValue::Axis(_) => 9,
            StateValue::Coord2(_) => 10,
            StateValue::Coord3(_) => 11,
        }
    }
}

impl Readable for StateValue {
    fn read_from<R: std::io::Read>(reader: &mut R) -> crate::prelude::VoxelResult<Self> {
        let id = u8::read_from(reader)?;
        Ok(match id {
            0 => StateValue::Null,
            1 => StateValue::Int(i64::read_from(reader)?),
            2 => StateValue::Bool(bool::read_from(reader)?),
            3 => StateValue::String(String::read_from(reader)?),
            4 => StateValue::Direction(Direction::read_from(reader)?),
            5 => StateValue::Cardinal(Cardinal::read_from(reader)?),
            6 => StateValue::Rotation(Rotation::read_from(reader)?),
            7 => StateValue::Flip(Flip::read_from(reader)?),
            8 => StateValue::Orientation(Orientation::read_from(reader)?),
            9 => StateValue::Axis(Axis::read_from(reader)?),
            10 => StateValue::Coord2(IVec2::read_from(reader)?),
            11 => StateValue::Coord3(IVec3::read_from(reader)?),
            _ => return Err(crate::prelude::VoxelError::InvalidBinaryFormat),
        })
    }
}

impl Writeable for StateValue {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> crate::prelude::VoxelResult<u64> {
        self.id().write_to(writer)?;
        Ok(match self {
            StateValue::Null => return Ok(1),
            StateValue::Int(value) => value.write_to(writer)?,
            StateValue::Bool(value) => value.write_to(writer)?,
            StateValue::String(value) => value.write_to(writer)?,
            StateValue::Direction(value) => value.write_to(writer)?,
            StateValue::Cardinal(value) => value.write_to(writer)?,
            StateValue::Rotation(value) => value.write_to(writer)?,
            StateValue::Flip(value) => value.write_to(writer)?,
            StateValue::Orientation(value) => value.write_to(writer)?,
            StateValue::Axis(value) => value.write_to(writer)?,
            StateValue::Coord2(value) => value.write_to(writer)?,
            StateValue::Coord3(value) => value.write_to(writer)?,
        } + 1)
    }
}

impl BlockState {
    pub fn new<S: StrToOwned, It: IntoIterator<Item = BlockProperty>>(name: S, properties: It) -> Self {
        let name = name.str_to_owned();
        let mut sorted_properties: Vec<_> = properties.into_iter().collect();
        sorted_properties.sort_by(|a, b| {
            a.name.cmp(&b.name)
        });
        Self {
            name,
            sorted_properties
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn properties(&self) -> &[BlockProperty] {
        &self.sorted_properties
    }

    pub fn set_property<S: AsRef<str>, St: Into<StateValue>>(&mut self, name: S, value: St) -> Option<StateValue> {
        let search = self.sorted_properties.binary_search_by(|prop| {
            let prop_name: &str = &prop.name;
            prop_name.cmp(name.as_ref())
        });
        match search {
            Ok(index) => {
                let mut old = value.into();
                std::mem::swap(&mut self.sorted_properties[index].value, &mut old);
                Some(old)
            }
            Err(index) => {
                self.sorted_properties.insert(index, BlockProperty::new(name.as_ref().to_owned(), value));
                None
            },
        }
    }

    pub fn get_property<S: AsRef<str>>(&self, name: S) -> Option<&StateValue> {
        let name = name.as_ref();
        if let Ok(index) = self.sorted_properties.binary_search_by(|prop| {
            let prop_name: &str = &prop.name;
            prop_name.cmp(name)
        }) {
            Some(&self.sorted_properties[index].value)
        } else {
            None
        }
    }

    /// Registers this [BlockState] with this block registry.
    pub fn register(&self) -> Id {
        blocks::register_state(self)
    }

    /// Finds the [BlockState] in the block registry.
    pub fn find(&self) -> Option<Id> {
        blocks::find_state(self)
    }
}

impl Readable for BlockState {
    fn read_from<R: std::io::Read>(reader: &mut R) -> crate::prelude::VoxelResult<Self> {
        let name = String::read_from(reader)?;
        let props_len = read_u24(reader)?;
        if props_len == 0 {
            return Ok(BlockState::new(name, []));
        }
        let sorted_properties: Vec<BlockProperty> = (0..props_len).map(|_| {
            let name = String::read_from(reader)?;
            let field = StateValue::read_from(reader)?;
            crate::core::error::Result::Ok(BlockProperty::new(name, field))
        }).try_collect()?;
        Ok(BlockState {
            name,
            sorted_properties
        })
    }
}

impl Writeable for BlockState {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<u64> {
        let mut length = self.name.write_to(writer)?;
        length += write_u24(writer, self.sorted_properties.len() as u32)?;
        self.sorted_properties.iter().try_fold(length, |mut length, prop| {
            length += prop.name.write_to(writer)?;
            Result::Ok(length + prop.value.write_to(writer)?)
        })
    }
}

impl<S: AsRef<str>> Index<S> for BlockState {
    type Output = StateValue;

    fn index(&self, index: S) -> &Self::Output {
        const NULL: StateValue = StateValue::Null;
        self.get_property(index).unwrap_or(&NULL)
    }
}

impl<S: AsRef<str>> IndexMut<S> for BlockState {
    fn index_mut(&mut self, index: S) -> &mut Self::Output {
        let name = index.as_ref();
        let search = self.sorted_properties.binary_search_by(|prop| {
            let prop_name: &str = &prop.name;
            prop_name.cmp(name)
        });
        match search {
            Ok(index) => {
                &mut self.sorted_properties[index].value
            }
            Err(index) => {
                self.sorted_properties.insert(index, BlockProperty::new(name, StateValue::Null));
                &mut self.sorted_properties[index].value
            }
        }
    }
}

impl From<()> for StateValue {
    fn from(value: ()) -> Self {
        StateValue::Null
    }
}

impl From<i64> for StateValue {
    fn from(value: i64) -> Self {
        StateValue::Int(value)
    }
}

impl From<bool> for StateValue {
    fn from(value: bool) -> Self {
        StateValue::Bool(value)
    }
}

impl<S: StrToOwned> From<S> for StateValue {
    fn from(value: S) -> Self {
        StateValue::String(value.str_to_owned())
    }
}

impl From<Direction> for StateValue {
    fn from(value: Direction) -> Self {
        StateValue::Direction(value)
    }
}

impl From<Cardinal> for StateValue {
    fn from(value: Cardinal) -> Self {
        StateValue::Cardinal(value)
    }
}

impl From<Rotation> for StateValue {
    fn from(value: Rotation) -> Self {
        StateValue::Rotation(value)
    }
}

impl From<Flip> for StateValue {
    fn from(value: Flip) -> Self {
        StateValue::Flip(value)
    }
}

impl From<Orientation> for StateValue {
    fn from(value: Orientation) -> Self {
        StateValue::Orientation(value)
    }
}

impl From<Coord> for StateValue {
    fn from(value: Coord) -> Self {
        StateValue::Coord3(value.into())
    }
}

impl From<IVec3> for StateValue {
    fn from(value: IVec3) -> Self {
        StateValue::Coord3(value)
    }
}

impl From<(i32, i32, i32)> for StateValue {
    fn from(value: (i32, i32, i32)) -> Self {
        StateValue::Coord3(value.into())
    }
}

impl From<ChunkCoord> for StateValue {
    fn from(value: ChunkCoord) -> Self {
        StateValue::Coord2(value.into())
    }
}

impl From<IVec2> for StateValue {
    fn from(value: IVec2) -> Self {
        StateValue::Coord2(value)
    }
}

impl From<(i32, i32)> for StateValue {
    fn from(value: (i32, i32)) -> Self {
        StateValue::Coord2(value.into())
    }
}

// impl From<Flip> for StateValue {
//     fn from(value: Flip) -> Self {
//         StateValue::Flip(value)
//     }
// }

impl From<Axis> for StateValue {
    fn from(value: Axis) -> Self {
        StateValue::Axis(value)
    }
}

impl BlockProperty {
    pub fn new<S: StrToOwned, St: Into<StateValue>>(name: S, value: St) -> Self {
        Self {
            name: name.str_to_owned(),
            value: value.into(),
        }
    }
}

#[macro_export]
macro_rules! blockstate {
    ($name:ident$(, $($prop_name:ident = $prop_value:expr),*$(,)?)?) => {
        $crate::core::voxel::blockstate::BlockState::new(stringify!($name), [
            $($(
                $crate::core::voxel::blockstate::BlockProperty::new(stringify!($prop_name), $prop_value),
            )*)?
        ])
    };
}

impl<B: Borrow<BlockState>> From<B> for Id {
    fn from(value: B) -> Self {
        blocks::register_state(value)
    }
}

#[test]
fn quick() {
    let state: Id = blockstate!(air).into();
}

impl std::fmt::Display for StateValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateValue::Null => write!(f, "null"),
            StateValue::Int(value) => write!(f, "{value}"),
            StateValue::Bool(value) => write!(f, "{value}"),
            StateValue::String(value) => {
                write!(f, "\"")?;
                value.chars().try_for_each(|c| {
                    match c {
                        '\r' => write!(f, "\\r"),
                        '\n' => write!(f, "\\n"),
                        '\\' => write!(f, "\\\\"),
                        '"' => write!(f, "\\\""),
                        '\t' => write!(f, "\\t"),
                        c => write!(f, "{c}"),
                    }
                })?;
                write!(f, "\"")
            },
            StateValue::Direction(direction) => match direction {
                Direction::NegX => write!(f, "NegX"),
                Direction::NegY => write!(f, "NegY"),
                Direction::NegZ => write!(f, "NegZ"),
                Direction::PosX => write!(f, "PosX"),
                Direction::PosY => write!(f, "PosY"),
                Direction::PosZ => write!(f, "PosZ"),
            },
            StateValue::Cardinal(cardinal) => match cardinal {
                Cardinal::West => write!(f, "West"),
                Cardinal::North => write!(f, "North"),
                Cardinal::East => write!(f, "East"),
                Cardinal::South => write!(f, "South"),
            },
            StateValue::Rotation(rotation) => {
                write!(f, "{rotation}")
            }
            StateValue::Flip(flip) => {
                write!(f, "{flip}")
            }
            StateValue::Orientation(orientation) => {
                write!(f, "{orientation}")
            }
            StateValue::Coord2(coord) => {
                write!(f, "({}, {})", coord.x, coord.y)
            }
            StateValue::Coord3(coord) => {
                write!(f, "({}, {}, {})", coord.x, coord.y, coord.z)
            }
            &StateValue::Axis(axis) => {
                write!(f, "Axis::{axis:?}")
            }
        }
    }
}

impl std::fmt::Display for BlockProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", &self.name, &self.value)
    }
}

impl std::fmt::Display for BlockState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())?;
        write!(f, "[")?;
        self.sorted_properties.iter().enumerate().try_for_each(|(i, prop)| {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{prop}")
        })?;
        write!(f, "]")
    }
}

impl BlockState {
    pub fn write_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[test]
fn print_blockstate_test() {
    let state = blockstate!(test, level = 13, text = "test\nblock", facing = Cardinal::East);
    println!("{state}");
}

#[test]
fn insert_test() {
    fn prop<S: StrToOwned, St: Into<StateValue>>(name: S, value: St) -> BlockProperty {
        BlockProperty::new(name, value)
    }
    let state = blockstate!(test, on = true, direction = Cardinal::East, level = 13, test = Direction::PosX);
    // let state = BlockState::new("test", [prop("on", true), prop("direction", Cardinal::East), prop("level", 13)]);
    println!("On --------: {:?}", state["on"]);
    println!("Direction -: {:?}", state["direction"]);
    println!("Level -----: {:?}", state["level"]);
    println!("test ------: {:?}", state["test"]);
    let dir = state.sorted_properties.binary_search_by(|a| { 
        let aname: &str = a.name.as_ref();
        aname.cmp("zfs")
    });
    match dir {
        Ok(index) => {
            println!("Found at {index}");
            println!("{}", state.sorted_properties[index].name);
        },
        Err(index) => println!("Insert at {index}"),
    }
    // let mut props = vec![
    //     prop("on", true),
    //     prop("direction", Cardinal::East),
    //     prop("level", 13),
    // ];
    // props.sort_by(|a, b| {
    //     a.name.cmp(&b.name)
    // });
    // for pp in props {
    //     println!("{}", pp.name);
    // }
    // let mut items = vec![1, 2, 3, 4];
    // let size = std::mem::size_of_val(&items);
    // println!("Size: {size}");
    // items.insert(2, 1234);
    // for item in items {
    //     println!("{item}");
    // }
}