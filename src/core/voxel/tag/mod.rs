use std::io::{Read, Write};

use hashbrown::HashMap;

use crate::core::{error::{Error, Result}, io::{Readable, Writeable}, math::{bit::*, coordmap::{Flip, Rotation}}, voxel::{axis::Axis, coord::Coord, direction::Direction, world::chunkcoord::ChunkCoord}};
use crate::core::io::*;
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
            [18 Cardinal      Byte      unbox   <crate::core::voxel::direction::Cardinal>   ]
            [19 Rotation      Byte      unbox   <crate::core::math::coordmap::Rotation>     ]
            [20 Axis          Byte      unbox   <crate::core::voxel::axis::Axis>            ]
            [21 Rgb           NonByte   unbox   <crate::core::voxel::rendering::color::Rgb> ]
            [22 Rgba          NonByte   unbox   <crate::core::voxel::rendering::color::Rgba>]
            [23 IVec2         NonByte   unbox   <bevy::math::IVec2>                         ]
            [24 IVec3         NonByte   unbox   <bevy::math::IVec3>                         ]
            [25 IVec4         NonByte   box     <bevy::math::IVec4>                         ]
            [26 Vec2          NonByte   unbox   <bevy::math::Vec2>                          ]
            [27 Vec3          NonByte   unbox   <bevy::math::Vec3>                          ]
            [28 Vec4          NonByte   box     <bevy::math::Vec4>                          ]
            [29 Mat2          NonByte   box     <bevy::math::Mat2>                          ]
            [30 Mat3          NonByte   box     <bevy::math::Mat3>                          ]
            [31 Mat4          NonByte   box     <bevy::math::Mat4>                          ]
            [32 Quat          NonByte   box     <bevy::math::Quat>                          ]
            [33 Bounds2       NonByte   box     <rollgrid::rollgrid2d::Bounds2D>            ]
            [34 Bounds3       NonByte   box     <rollgrid::rollgrid3d::Bounds3D>            ]
            [35 String        NonByte   box     <String>                                    ]
            [36 Array         NonByte   box     <crate::core::voxel::tag::Array>            ]
            [37 Map           NonByte   box     <hashbrown::HashMap<String, Tag>>           ]
            /* This line should remain commented out. It is a representation of what I wrote manually
            [38 Tag           NonByte   box     <Tag>                                       ]
            Continue writing new rows at index 39
            */
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
            Tag(Vec<Tag>) = 38,
        }

        impl Array {
            pub fn id(&self) -> u8 {
                match self {
                    Array::Empty => 0,
                    $(
                        Array::$name(_) => $id,
                    )*
                    Array::Tag(_) => 38,
                }
            }

            pub fn len(&self) -> usize {
                match self {
                    Array::Empty => 0,
                    $(
                        Array::$name(array) => array.len(),
                    )*
                    Array::Tag(array) => array.len(),
                }
            }
        }

        $(
            impl From<Vec<$type>> for Array {
                fn from(value: Vec<$type>) -> Self {
                    Array::$name(value)
                }
            }

            impl<const SIZE: usize> From<[$type; SIZE]> for Array {
                fn from(value: [$type; SIZE]) -> Self {
                    Array::$name(value.into())
                }
            }

            impl From<Vec<$type>> for Tag {
                fn from(value: Vec<$type>) -> Self {
                    Tag::Array(Box::new(value.into()))
                }
            }

            impl<const SIZE: usize> From<[$type; SIZE]> for Tag {
                fn from(value: [$type; SIZE]) -> Self {
                    Tag::Array(Box::new(value.into()))
                }
            }
        )*

        impl Readable for Array {
            fn read_from<R: Read>(mut reader: &mut R) -> Result<Self> {
                let id: u8 = u8::read_from(reader)?;
                Ok(match id {
                    0 => Array::Empty,
                    $(
                        $id => Array::$name(Vec::<$type>::read_from(reader)?),
                    )*
                    38 => Array::Tag(Vec::<Tag>::read_from(reader)?),
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
                    Array::Tag(array) => array.write_to(writer)?,
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

impl NonByte for Tag {}
impl From<Vec<Tag>> for Array {
    fn from(value: Vec<Tag>) -> Self {
        Array::Tag(value)
    }
}

impl From<Vec<Tag>> for Tag {
    fn from(value: Vec<Tag>) -> Self {
        Tag::Array(Box::new(value.into()))
    }
}

impl<const SIZE: usize> From<[Tag; SIZE]> for Array {
    fn from(value: [Tag; SIZE]) -> Self {
        Array::Tag(value.into())
    }
}

impl<const SIZE: usize> From<[Tag; SIZE]> for Tag {
    fn from(value: [Tag; SIZE]) -> Self {
        Tag::Array(Box::new(value.into()))
    }
}

impl<'a> From<Vec<&'a str>> for Array {
    fn from(value: Vec<&'a str>) -> Self {
        Array::String(value.into_iter().map(str::to_owned).collect())
    }
}

impl<'a, const SIZE: usize> From<[&'a str; SIZE]> for Array {
    fn from(value: [&'a str; SIZE]) -> Self {
        Array::String(value.into_iter().map(str::to_owned).collect())
    }
}

impl<'a> From<Vec<&'a str>> for Tag {
    fn from(value: Vec<&'a str>) -> Self {
        Tag::Array(Box::new(Array::String(value.into_iter().map(str::to_owned).collect())))
    }
}

impl<'a, const SIZE: usize> From<[&'a str; SIZE]> for Tag {
    fn from(value: [&'a str; SIZE]) -> Self {
        Tag::Array(Box::new(Array::String(value.into_iter().map(str::to_owned).collect())))
    }
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