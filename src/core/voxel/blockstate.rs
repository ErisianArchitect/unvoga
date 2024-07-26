#![allow(unused)]
use std::{borrow::Borrow, fmt::Debug, ops::{Index, IndexMut}};

use bevy::math::{IVec2, IVec3};
use itertools::Itertools;
use crate::core::error::*;
use crate::prelude::*;

use crate::core::util::traits::StrToOwned;

use super::{axis::Axis, blocks::{self, Id}, coord::Coord, direction::{Cardinal, Direction}, faceflags::FaceFlags, world::chunkcoord::ChunkCoord};
use super::statevalue::StateValue;

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