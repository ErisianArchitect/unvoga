#![allow(unused)]
use bevy::math::Vec3;

use crate::prelude::*;

use super::{coordmap::{pack_flip_and_rotation, unpack_flip_and_rotation}, flip::Flip, maptable::{self, map_face_coord_table_index}, rotation::Rotation};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Orientation {
    pub flip: Flip,
    pub rotation: Rotation,
}

impl Orientation {
    pub const fn new(rotation: Rotation, flip: Flip) -> Self {
        Self {
            flip,
            rotation
        }
    }
    
    /// Packs the flip and rotation into a single byte where the first 3 bits are the flip
    /// and the remaining 5 bits are the rotation.
    pub fn pack(self) -> u8 {
        pack_flip_and_rotation(self.flip, self.rotation)
    }

    /// Unpacks the flip and rotation from a single byte where the first 3 bits are the flip
    /// and the remaining 5 bits are the rotation.
    pub fn unpack(packed: u8) -> Self {
        let (flip, rotation) = unpack_flip_and_rotation(packed);
        Self {
            flip,
            rotation
        }
    }

    /// `reface` can be used to determine where a face will end up after orientation.
    /// First rotates and then flips the face.
    pub fn reface(self, face: Direction) -> Direction {
        let rotated = self.rotation.reface(face);
        rotated.flip(self.flip)
    }

    /// This determines which face was oriented to `face`. I hope that makes sense.
    pub fn source_face(self, face: Direction) -> Direction {
        let flipped = face.flip(self.flip);
        self.rotation.source_face(flipped)
    }

    /// If you're using this function to transform mesh vertices, make sure that you 
    /// change your indices if the face will be flipped (for backface culling)
    pub fn transform<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T, T)> + From<(T, T, T)>>(self, point: C) -> C {
        let rotated = self.rotation.rotate(point);
        self.flip.flip_coord(rotated)
    }

    /// To get the most benefit out of this, it is advised that you center your coords around (0, 0).
    /// So if you're trying to map a coord within a rect of size (16, 16), you would subtract 8 from the
    /// x and y of the coord, then pass that offset coord to this function, then add 8 back to the x and y
    /// to get your final coord.
    pub fn source_face_coord<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T)> + From<(T, T)>>(self, face: Direction, uv: C) -> C {
        // I actually realized that I did this backwards. For what I want, I need to figure out the source coord.
        let table_index = map_face_coord_table_index(self.rotation, self.flip, face);
        let coordmap = maptable::MAP_COORD_TABLE[table_index];
        coordmap.map(uv)
    }

    // pub fn source_face_coord<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T)> + From<(T, T)>>(self, face: Direction, uv: C) -> C {
    //     let table_index = map_face_coord_table_index(self.rotation, self.flip, face);
    //     let coordmap = maptable::SOURCE_FACE_COORD_TABLE[table_index];
    //     // todo!("This method doesn't work properly.");
    //     coordmap.map(uv)
    // }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AxisMap {
    PosX,
    PosY,
    NegX,
    NegY,
}

impl AxisMap {
    pub fn map<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T)>>(self, coord: C) -> T {
        let (x, y): (T, T) = coord.into();
        match self {
            AxisMap::PosX => x,
            AxisMap::PosY => y,
            AxisMap::NegX => -x,
            AxisMap::NegY => -y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CoordMap {
    x: AxisMap,
    y: AxisMap,
}

impl CoordMap {
    pub const fn new(x: AxisMap, y: AxisMap) -> Self {
        Self {x, y}
    }
    pub fn map<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T)> + From<(T, T)>>(self, coord: C) -> C {
        let coord: (T, T) = coord.into();
        let coord = (self.x.map(coord), self.y.map(coord));
        C::from(coord)
    }
}

// I used this to generate the table in maptable.rs and I don't need it anymore, but I'm going
// to keep it around just in case.
// fn map_face_coord_naive(orientation: Orientation, face: Direction) -> CoordMap {
//     // First I will attempt a naive implementation, then I will use the naive implementation to generate code
//     // for a more optimized implementation.
//     // First get the source face
//     let source_face = orientation.source_face(face);
//     // next, get the up, right, down, and left for the source face and arg face.
//     let src_up = source_face.up();
//     let src_right = source_face.right();
//     let src_down = source_face.down();
//     let src_left = source_face.left();
//     let face_up = face.up();
//     let face_right = face.right();
//     let face_down = face.down();
//     let face_left = face.left();
//     // Next, reface the src_dir faces
//     let rsrc_up = orientation.reface(src_up);
//     let rsrc_right = orientation.reface(src_right);
//     let rsrc_down = orientation.reface(src_down);
//     let rsrc_left = orientation.reface(src_left);
//     // Now match up the faces
//     let x_map = if face_right == rsrc_right {
//         AxisMap::PosX
//     } else if face_right == rsrc_up {
//         AxisMap::NegY
//     } else if face_right == rsrc_left {
//         AxisMap::NegX
//     } else {
//         AxisMap::PosY
//     };
//     let y_map = if face_up == rsrc_up {
//         AxisMap::PosY
//     } else if face_up == rsrc_left {
//         AxisMap::PosX
//     } else if face_up == rsrc_down {
//         AxisMap::NegY
//     } else {
//         AxisMap::NegX
//     };
//     CoordMap {
//         x: x_map,
//         y: y_map
//     }
// }
fn source_face_coord_naive(orientation: Orientation, face: Direction) -> CoordMap {
    // First I will attempt a naive implementation, then I will use the naive implementation to generate code
    // for a more optimized implementation.
    // First get the source face
    let source_face = orientation.source_face(face);
    // next, get the up, right, down, and left for the source face and arg face.
    let src_up = source_face.up();
    let src_right = source_face.right();
    let src_down = source_face.down();
    let src_left = source_face.left();
    let face_up = face.up();
    let face_right = face.right();
    let face_down = face.down();
    let face_left = face.left();
    // Next, reface the src_dir faces
    let rsrc_up = orientation.reface(src_up);
    let rsrc_right = orientation.reface(src_right);
    let rsrc_down = orientation.reface(src_down);
    let rsrc_left = orientation.reface(src_left);
    // Now match up the faces
    let x_map = if face_right == rsrc_right {
        AxisMap::PosX
    } else if face_right == rsrc_up {
        AxisMap::PosY
    } else if face_right == rsrc_left {
        AxisMap::NegX
    } else {
        AxisMap::NegY
    };
    let y_map = if face_up == rsrc_up {
        AxisMap::PosY
    } else if face_up == rsrc_left {
        AxisMap::NegX
    } else if face_up == rsrc_down {
        AxisMap::NegY
    } else {
        AxisMap::PosX
    };
    CoordMap {
        x: x_map,
        y: y_map
    }
}

#[cfg(test)]
mod testing_sandbox {
    use bevy::math::vec2;

    #[test]
    fn check_solution() {
        let orientation = Orientation::new(Rotation::new(Direction::PosX, 2), Flip::XY);
        let face = Direction::NegZ;
        // let coordmap = map_face_coord_naive(orientation, face);
        // let table_index = maptable::map_face_coord_table_index(orientation.rotation, orientation.flip, face);
        // let table_map = maptable::MAP_COORD_TABLE[table_index];
        // assert_eq!(coordmap, table_map);
        let coord = (-1, -2);
        // let mapped = orientation.transform(coord);
        // println!("{coord:?} {mapped:?}");
        let mapped = orientation.source_face_coord(face, coord);
        // let unmapped = orientation.source_face_coord(face, coord);
        println!("{coord:?} {mapped:?}");
    }

    // This is used to generate the table in maptable.rs.
    // you need to uncoment map_face_coord_naive for this to work.
    // I commented it out because I don't need it anymore, but I'd like to keep
    // the code around in case I need it later as a reference.
    // #[test]
    // fn map_coord_gencode() {
    //     const fn map_axismap(a: AxisMap) -> &'static str {
    //         match a {
    //             AxisMap::PosX => "x",
    //             AxisMap::PosY => "y",
    //             AxisMap::NegX => "-x",
    //             AxisMap::NegY => "-y",
    //         }
    //     }
    //     let output = {
    //         use std::fmt::Write;
    //         let mut output = String::new();
    //         let mut count = 0usize;
    //         for flipi in 0..8 { // flip
    //             for roti in 0..24 { // rotation
    //                 Direction::iter_index_order().for_each(|face| {
    //                     count += 1;
    //                     let map = map_face_coord_naive(Orientation::new(Rotation(roti as u8), Flip(flipi as u8)), face);
    //                     // println!("({flipi}, {roti}, {face})");
    //                     writeln!(output, "CoordMap::new(AxisMap::{:?}, AxisMap::{:?}),", map.x, map.y);
    //                 });
    //             }
    //         }
    //         output
    //     };
    //     use std::io::{Write, BufWriter};
    //     use std::fs::File;
    //     let mut writer = BufWriter::new(File::create("ignore/map_coord_table.rs").expect("Failed to open file"));
    //     writer.write_all(output.as_bytes());
    //     println!("Wrote the output to file at ./ignore/map_coord_table.rs");
    // }
    // #[test]
    // fn source_coord_gencode() {
    //     const fn map_axismap(a: AxisMap) -> &'static str {
    //         match a {
    //             AxisMap::PosX => "x",
    //             AxisMap::PosY => "y",
    //             AxisMap::NegX => "-x",
    //             AxisMap::NegY => "-y",
    //         }
    //     }
    //     let output = {
    //         use std::fmt::Write;
    //         let mut output = String::new();
    //         let mut count = 0usize;
    //         for flipi in 0..8 { // flip
    //             for roti in 0..24 { // rotation
    //                 Direction::iter_index_order().for_each(|face| {
    //                     count += 1;
    //                     let map = source_face_coord_naive(Orientation::new(Rotation(roti as u8), Flip(flipi as u8)), face);
    //                     // println!("({flipi}, {roti}, {face})");
    //                     writeln!(output, "CoordMap::new(AxisMap::{:?}, AxisMap::{:?}),", map.x, map.y);
    //                 });
    //             }
    //         }
    //         output
    //     };
    //     use std::io::{Write, BufWriter};
    //     use std::fs::File;
    //     let mut writer = BufWriter::new(File::create("ignore/source_face_coord_table.rs").expect("Failed to open file"));
    //     writer.write_all(output.as_bytes());
    //     println!("Wrote the output to file at ./ignore/source_face_coord_table.rs");
    // }

    use crate::core::math::maptable;

    use super::*;
    #[test]
    fn sandbox() {
        // let coord = Orientation::default().map_face_coord(Direction::NegX, (3, 5));
        // let orientation = Orientation::new(Rotation::new(Direction::NegY, 1), Flip::X | Flip::Y | Flip::Z);
        // let face = Direction::NegY;
        // let coordmap = map_face_coord_naive(orientation, face);
        // println!("Expect (NegY, NegX) got {coordmap:?}");
    }
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Orientation({},{})", self.flip, self.rotation)
    }
}