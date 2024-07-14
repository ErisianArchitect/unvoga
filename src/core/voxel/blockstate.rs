use std::{borrow::Borrow, fmt::Debug, ops::{Index, IndexMut}};

use crate::core::{math::coordmap::{Flip, Rotation}, util::traits::StrToOwned};

use super::{axis::Axis, blocks::{self, StateRef}, coord::Coord, direction::{Cardinal, Direction}, world::chunkcoord::ChunkCoord};

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
pub enum StateValue {
    #[default]
    Null,
    Int(i64),
    Bool(bool),
    String(String),
    Direction(Direction),
    Cardinal(Cardinal),
    Rotation(Rotation),
    Flip(Flip),
    Axis(Axis),
    Coord2(ChunkCoord),
    Coord3(Coord),
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
    pub fn register(&self) -> StateRef {
        blocks::register_state(self)
    }

    /// Finds the [BlockState] in the block registry.
    pub fn find(&self) -> Option<StateRef> {
        blocks::find_state(self)
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

impl From<Coord> for StateValue {
    fn from(value: Coord) -> Self {
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
        StateValue::Coord2(value)
    }
}

impl From<(i32, i32)> for StateValue {
    fn from(value: (i32, i32)) -> Self {
        StateValue::Coord2(value.into())
    }
}

impl From<Flip> for StateValue {
    fn from(value: Flip) -> Self {
        StateValue::Flip(value)
    }
}

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

impl<B: Borrow<BlockState>> From<B> for StateRef {
    fn from(value: B) -> Self {
        blocks::register_state(value)
    }
}

#[test]
fn quick() {
    let state: StateRef = blockstate!(air).into();
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
                write!(f, "Rotation(up={:?}, forward={:?}, angle={})", rotation.up(), rotation.forward(), rotation.angle())
            }
            StateValue::Coord2(coord) => {
                write!(f, "({}, {})", coord.x, coord.z)
            }
            StateValue::Coord3(coord) => {
                write!(f, "({}, {}, {})", coord.x, coord.y, coord.z)
            }
            &StateValue::Flip(flip) => {
                write!(f, "Flip::")?;
                if flip == Flip::NONE {
                    write!(f, "None")?;
                } else {
                    if flip.x() {
                        write!(f, "X")?;
                    }
                    if flip.y() {
                        write!(f, "Y")?;
                    }
                    if flip.z() {
                        write!(f, "Z")?;
                    }
                }
                Ok(())
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