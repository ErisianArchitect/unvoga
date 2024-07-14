use std::io::{Read, Write};

use hashbrown::HashMap;

use crate::core::{error::{Error, Result}, io::{Readable, Writeable}, math::{bit::*, coordmap::{Flip, Rotation}}, voxel::{axis::Axis, coord::Coord, direction::Direction, world::chunkcoord::ChunkCoord}};

pub trait NonByte {}
pub trait Byte {}
macro_rules! tag_table {
    ($macro:path) => {
        $macro! {
            //
            [ 1 Bool          Byte      unbox   <bool>                                      ]
            [ 2 BitFlags8     Byte      unbox   <crate::core::math::bit::BitFlags8>         ]
            [ 3 BitFlags16    NonByte   unbox   <crate::core::math::bit::BitFlags16>        ]
            [ 4 BitFlags32    NonByte   unbox   <crate::core::math::bit::BitFlags32>        ]
            [ 5 BitFlags64    NonByte   unbox   <crate::core::math::bit::BitFlags64>        ]
            [ 6 BitFlags128   NonByte   box     <crate::core::math::bit::BitFlags128>       ]
            [ 7 U8            Byte      unbox   <u8>                                        ]
            [ 8 I8            Byte      unbox   <i8>                                        ]
            [ 9 U16           NonByte   unbox   <u16>                                       ]
            [10 I16           NonByte   unbox   <i16>                                       ]
            [11 U32           NonByte   unbox   <u32>                                       ]
            [12 I32           NonByte   unbox   <i32>                                       ]
            [13 U64           NonByte   unbox   <u64>                                       ]
            [14 I64           NonByte   unbox   <i64>                                       ]
            [15 F32           NonByte   unbox   <f32>                                       ]
            [16 F64           NonByte   unbox   <f64>                                       ]
            [17 Direction     Byte      unbox   <crate::core::voxel::direction::Direction>  ]
            [18 Rotation      Byte      unbox   <crate::core::math::coordmap::Rotation>     ]
            [19 Axis          Byte      unbox   <crate::core::voxel::axis::Axis>            ]
            [20 Rgb           NonByte   unbox   <crate::core::voxel::rendering::color::Rgb> ]
            [21 Rgba          NonByte   unbox   <crate::core::voxel::rendering::color::Rgba>]
            [22 IVec2         NonByte   unbox   <bevy::math::IVec2>                         ]
            [23 IVec3         NonByte   unbox   <bevy::math::IVec3>                         ]
            [24 IVec4         NonByte   box     <bevy::math::IVec4>                         ]
            [25 Vec2          NonByte   unbox   <bevy::math::Vec2>                          ]
            [26 Vec3          NonByte   unbox   <bevy::math::Vec3>                          ]
            [27 Vec4          NonByte   box     <bevy::math::Vec4>                          ]
            [28 Mat2          NonByte   box     <bevy::math::Mat2>                          ]
            [29 Mat3          NonByte   box     <bevy::math::Mat3>                          ]
            [30 Mat4          NonByte   box     <bevy::math::Mat4>                          ]
            [31 Quat          NonByte   box     <bevy::math::Quat>                          ]
            [32 Bounds2       NonByte   box     <rollgrid::rollgrid2d::Bounds2D>            ]
            [33 Bounds3       NonByte   box     <rollgrid::rollgrid3d::Bounds3D>            ]
            [34 String        NonByte   box     <String>                                    ]
            [35 Array         NonByte   box     <crate::core::voxel::tag::Array>            ]
            [36 Map           NonByte   box     <hashbrown::HashMap<String, Tag>>           ]
        }
    };
}

macro_rules! table_impls {
    ($([$id:literal $name:ident $impl:ident $box:ident <$type:ty> $($end:tt)*])*) => {
        // Blanket impls
        $(
            impl $impl for $type {}
        )*

        #[derive(Debug, Default, Clone, PartialEq)]
        #[repr(u8)]
        pub enum Tag {
            #[default]
            Null = 0,
            $(
                $name(table_impls!(@box_unbox: $box $type)) = $id,
            )*
        }

        impl Tag {
            pub fn id(&self) -> u8 {
                match self {
                    Tag::Null => 0,
                    $(
                        Tag::$name(_) => $id,
                    )*
                }
            }
        }

        impl Readable for Tag {
            fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
                let id: u8 = u8::read_from(reader)?;
                Ok(match id {
                    0 => Tag::Null,
                    $(
                        $id => Tag::$name(<table_impls!(@box_unbox: $box $type)>::read_from(reader)?),
                    )*
                    _ => return Err(Error::InvalidBinaryFormat),
                })
            }
        }

        impl Writeable for Tag {
            fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
                self.id().write_to(writer)?;
                Ok(match self {
                    Tag::Null => 0,
                    $(
                        Tag::$name(value) => value.write_to(writer)?,
                    )*
                } + 1)
            }
        }

        #[derive(Debug, Default, Clone, PartialEq)]
        #[repr(u8)]
        pub enum Array {
            #[default]
            Empty = 0,
            $(
                $name(Vec<$type>) = $id,
            )*
        }

        impl Array {
            pub fn id(&self) -> u8 {
                match self {
                    Array::Empty => 0,
                    $(
                        Array::$name(_) => $id,
                    )*
                }
            }

            pub fn len(&self) -> usize {
                match self {
                    Array::Empty => 0,
                    $(
                        Array::$name(array) => array.len(),
                    )*
                }
            }
        }

        impl Readable for Array {
            fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
                let id: u8 = u8::read_from(reader)?;
                Ok(match id {
                    0 => Array::Empty,
                    $(
                        $id => Array::$name(Vec::<$type>::read_from(reader)?),
                    )*
                    _ => return Err(Error::InvalidBinaryFormat),
                })
            }
        }
        
        impl Writeable for Array {
            fn write_to<W: Write>(&self, mut writer: &mut W) -> Result<u64> {
                self.id().write_to(writer)?;
                Ok(match self {
                    Array::Empty => 0,
                    $(
                        Array::$name(array) => array.write_to(writer)?,
                    )*
                } + 1)
            }
        }
    };
    (@box_unbox: box $type:ty) => {
        Box<$type>
    };
    (@box_unbox: unbox $type:ty) => {
        $type
    };
}

tag_table!(table_impls);

macro_rules! from_impls {
    ($([$id:literal $name:ident $impl:ident $box:ident <$type:ty> $($end:tt)*])*) => {
        $(
            from_impls!($box $name $type);
        )*
    };
    (box $name:ident $type:ty) => {
        impl From<Box<$type>> for Tag {
            fn from(value: Box<$type>) -> Tag {
                Tag::$name(value)
            }
        }
        impl From<$type> for Tag {
            fn from(value: $type) -> Tag {
                Tag::$name(Box::new(value))
            }
        }
    };
    (unbox $name:ident $type:ty) => {
        impl From<$type> for Tag {
            fn from(value: $type) -> Tag {
                Tag::$name(value)
            }
        }
    };
}

tag_table!(from_impls);

impl Tag {
    pub const NULL: Tag = Tag::Null;

    pub fn is_null(&self) -> bool {
        matches!(self, Tag::Null)
    }
}

impl From<&str> for Tag {
    fn from(value: &str) -> Self {
        Tag::String(Box::new(value.to_owned()))
    }
}

impl<S: AsRef<str>> std::ops::Index<S> for Tag {
    type Output = Tag;
    fn index(&self, index: S) -> &Self::Output {
        const NULL: Tag = Tag::Null;
        let Tag::Map(map) = self else {
            return &NULL;
        };
        map.get(index.as_ref()).unwrap_or(&NULL)
    }
}

impl<S: AsRef<str>> std::ops::IndexMut<S> for Tag {
    fn index_mut(&mut self, index: S) -> &mut Self::Output {
        if let Tag::Map(map) = self {
            map.entry(index.as_ref().to_owned()).or_insert(Tag::Null)
        } else {
            panic!("Not a map");
        }
    }
}

#[test]
fn types() {
    use bevy::math::*;
    let tag = Tag::default();
    println!("{tag:?}");
    // from_impls!(@box_impl: Map Box<HashMap<String, Tag>>);
    let tag = Tag::from(hashbrown::HashMap::new());
    println!("{tag:?}");
    
}