#![allow(unused)]
use bevy::math::Vec3;

use crate::prelude::*;

use super::{coordmap::{pack_flip_and_rotation, unpack_flip_and_rotation}, flip::Flip, orient_table::{self, map_face_coord_table_index}, rotation::Rotation};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Orientation {
    pub flip: Flip,
    pub rotation: Rotation,
}

use Direction::*;
impl Orientation {
    pub const UNORIENTED: Orientation = Orientation::new(Rotation::new(PosY, 0), Flip::NONE);
    pub const ROTATE_X: Rotation = Rotation::new(Direction::NegZ, 2);
    pub const X_ROTATIONS: [Rotation; 4] = [
        Rotation::new(PosY, 0),
        Rotation::new(NegZ, 2),
        Rotation::new(NegY, 0),
        Rotation::new(PosZ, 0),
    ];
    pub const ROTATE_Y: Rotation = Rotation::new(Direction::PosY, 1);
    pub const Y_ROTATIONS: [Rotation; 4] = [
        Rotation::new(PosY, 0),
        Rotation::new(PosY, 1),
        Rotation::new(PosY, 2),
        Rotation::new(PosY, 3),
    ];
    pub const ROTATE_Z: Rotation = Rotation::new(Direction::PosX, 1);
    pub const Z_ROTATIONS: [Rotation; 4] = [
        Rotation::new(PosY, 0),
        Rotation::new(PosX, 1),
        Rotation::new(NegY, 2),
        Rotation::new(NegX, 3),
    ];
    const CORNER_ROTATIONS_MATRIX: [[[Rotation; 2]; 2]; 2] = [
        [
            [Rotation::new(PosZ, 3), Rotation::new(NegX, 2)],
            [Rotation::new(PosX, 0), Rotation::new(NegZ, 1)]
        ],
        [
            [Rotation::new(NegX, 0), Rotation::new(NegZ, 3)],
            [Rotation::new(PosZ, 1), Rotation::new(PosX, 2)]
        ],
    ];
    // pub const CORNER_X0_Y0_Z0_ROTATIONS: [Rotation; 4] = [
    //     Rotation::
    // ];

    pub const fn new(rotation: Rotation, flip: Flip) -> Self {
        Self {
            flip,
            rotation
        }
    }
    
    /// Packs the flip and rotation into a single byte where the first 3 bits are the flip
    /// and the remaining 5 bits are the rotation.
    pub const fn pack(self) -> u8 {
        pack_flip_and_rotation(self.flip, self.rotation)
    }

    /// Unpacks the flip and rotation from a single byte where the first 3 bits are the flip
    /// and the remaining 5 bits are the rotation.
    pub const fn unpack(packed: u8) -> Self {
        let (flip, rotation) = unpack_flip_and_rotation(packed);
        Self {
            flip,
            rotation
        }
    }

    /// Gets the direction that [Direction::PosY] is pointing towards.
    pub const fn up(self) -> Direction {
        self.reface(Direction::PosY)
    }

    /// Gets the direction that [Direction::NegY] is pointing towards.
    pub const fn down(self) -> Direction {
        self.reface(Direction::NegY)
    }

    /// Gets the direction that [Direction::NegZ] is pointing towards.
    pub const fn forward(self) -> Direction {
        self.reface(Direction::NegZ)
    }

    /// Gets the direction that [Direction::PosZ] is pointing towards.
    pub const fn backward(self) -> Direction {
        self.reface(Direction::PosZ)
    }

    /// Gets the direction that [Direction::NegX] is pointing towards.
    pub const fn left(self) -> Direction {
        self.reface(Direction::NegX)
    }

    /// Gets the direction that [Direction::PosX] is pointing towards.
    pub const fn right(self) -> Direction {
        self.reface(Direction::PosX)
    }

    /// `reface` can be used to determine where a face will end up after orientation.
    /// First rotates and then flips the face.
    pub const fn reface(self, face: Direction) -> Direction {
        let rotated = self.rotation.reface(face);
        rotated.flip(self.flip)
    }

    /// This determines which face was oriented to `face`. I hope that makes sense.
    pub const fn source_face(self, face: Direction) -> Direction {
        let flipped = face.flip(self.flip);
        self.rotation.source_face(flipped)
    }

    /// If you're using this method to transform mesh vertices, make sure that you 
    /// reverse your indices if the face will be flipped (for backface culling). To
    /// determine if your indices need to be inverted, simply XOR each axis of the [Orientation]'s [Flip].
    /// This method will rotate and then flip the coordinate.
    pub fn transform<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T, T)> + From<(T, T, T)>>(self, point: C) -> C {
        let rotated = self.rotation.rotate(point);
        self.flip.flip_coord(rotated)
    }

    /// This method can tell you where on the target face a source UV is.
    /// To get the most benefit out of this, it is advised that you center your coords around (0, 0).
    /// So if you're trying to map a coord within a rect of size (16, 16), you would subtract 8 from the
    /// x and y of the coord, then pass that offset coord to this function, then add 8 back to the x and y
    /// to get your final coord.
    pub fn map_face_coord<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T)> + From<(T, T)>>(self, face: Direction, uv: C) -> C {
        let table_index = map_face_coord_table_index(self.rotation, self.flip, face);
        let coordmap = orient_table::MAP_COORD_TABLE[table_index];
        coordmap.map(uv)
    }

    /// This method can tell you where on the source face a target UV is.
    /// To get the most benefit out of this, it is advised that you center your coords around (0, 0).
    /// So if you're trying to map a coord within a rect of size (16, 16), you would subtract 8 from the
    /// x and y of the coord, then pass that offset coord to this function, then add 8 back to the x and y
    /// to get your final coord.
    pub fn source_face_coord<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T)> + From<(T, T)>>(self, face: Direction, uv: C) -> C {
        let table_index = map_face_coord_table_index(self.rotation, self.flip, face);
        let coordmap = orient_table::SOURCE_FACE_COORD_TABLE[table_index];
        coordmap.map(uv)
    }

    /// Apply an orientation to an orientation.
    pub const fn reorient(self, orientation: Orientation) -> Self {
        let up = self.up();
        let fwd = self.forward();
        let reup = orientation.reface(up);
        let refwd = orientation.reface(fwd);
        let flip = self.flip.flip(orientation.flip);
        let flipup = reup.flip(flip);
        let flipfwd = refwd.flip(flip);
        let Some(rot) = Rotation::from_up_and_forward(flipup, flipfwd) else {
            unreachable!()
        };
        Orientation::new(rot, flip)
    }

    /// Remove an orientation from an orientation.
    /// This is the inverse operation to [Orientation::reorient].
    pub const fn deorient(self, orientation: Orientation) -> Self {
        let up = self.up();
        let fwd = self.forward();
        let reup = orientation.source_face(up);
        let refwd = orientation.source_face(fwd);
        let flip = self.flip.flip(orientation.flip);
        let flipup = reup.flip(flip);
        let flipfwd = refwd.flip(flip);
        let Some(rot) = Rotation::from_up_and_forward(flipup, flipfwd) else {
            unreachable!()
        };
        Orientation::new(rot, flip)
    }
    
    /// Returns the orientation that can be applied to deorient by [self].
    pub const fn invert(self) -> Self {
        Orientation::UNORIENTED.deorient(self)
    }

    pub const fn flip_x(mut self) -> Self {
        self.flip = self.flip.flip_x();
        self
    }

    pub const fn flip_y(mut self) -> Self {
        self.flip = self.flip.flip_y();
        self
    }

    pub const fn flip_z(mut self) -> Self {
        self.flip = self.flip.flip_z();
        self
    }

    pub const fn rotate_x(self, angle: i32) -> Self {
        self.reorient(Orientation::new(
            Orientation::X_ROTATIONS[angle.rem_euclid(4) as usize],
            Flip::NONE
        ))
    }

    pub const fn rotate_y(self, angle: i32) -> Self {
        self.reorient(Orientation::new(
            Orientation::Y_ROTATIONS[angle.rem_euclid(4) as usize],
            Flip::NONE
        ))
    }

    pub const fn rotate_z(self, angle: i32) -> Self {
        self.reorient(Orientation::new(
            Orientation::Z_ROTATIONS[angle.rem_euclid(4) as usize],
            Flip::NONE
        ))
    }
}

impl From<Rotation> for Orientation {
    fn from(value: Rotation) -> Self {
        Orientation::new(value, Flip::NONE)
    }
}

impl From<Flip> for Orientation {
    fn from(value: Flip) -> Self {
        Orientation::new(Rotation::default(), value)
    }
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn all_rotations() -> impl Iterator<Item = Rotation> {
        Direction::iter().flat_map(|up| (0..4).map(move |angle| Rotation::new(up, angle)))
    }

    fn all_orientations() -> impl Iterator<Item = Orientation> {
        all_rotations().flat_map(|rotation| {
            (0..8).map(move |flip| Orientation::new(rotation, Flip(flip)))
        })
    }

    // I used this to generate the table in maptable.rs and I don't need it anymore, but I'm going
    // to keep it around just in case.
    fn map_face_coord_naive(orientation: Orientation, face: Direction) -> CoordMap {
        // First I will attempt a naive implementation, then I will use the naive implementation to generate code
        // for a more optimized implementation.
        // First get the source face
        let source_face = orientation.source_face(face);
        // next, get the up, right, down, and left for the source face and arg face.
        let face_up = face.up();
        let face_right = face.right();
        let src_up = source_face.up();
        let src_right = source_face.right();
        let src_down = source_face.down();
        let src_left = source_face.left();
        // Next, reface the src_dir faces
        let rsrc_up = orientation.reface(src_up);
        let rsrc_right = orientation.reface(src_right);
        let rsrc_down = orientation.reface(src_down);
        let rsrc_left = orientation.reface(src_left);
        // Now match up the faces
        let x_map = if face_right == rsrc_right {
            AxisMap::PosX
        } else if face_right == rsrc_up {
            AxisMap::NegY
        } else if face_right == rsrc_left {
            AxisMap::NegX
        } else {
            AxisMap::PosY
        };
        let y_map = if face_up == rsrc_up {
            AxisMap::PosY
        } else if face_up == rsrc_left {
            AxisMap::PosX
        } else if face_up == rsrc_down {
            AxisMap::NegY
        } else {
            AxisMap::NegX
        };
        CoordMap {
            x: x_map,
            y: y_map
        }
    }

    fn source_face_coord_naive(orientation: Orientation, face: Direction) -> CoordMap {
        // First I will attempt a naive implementation, then I will use the naive implementation to generate code
        // for a more optimized implementation.
        // First get the source face
        let source_face = orientation.source_face(face);
        // next, get the up, right, down, and left for the source face and arg face.
        let src_up = source_face.up();
        let src_right = source_face.right();
        let face_up = face.up();
        let face_right = face.right();
        let face_down = face.down();
        let face_left = face.left();
        // Next, reface the src_dir faces
        let rsrc_up = orientation.reface(src_up);
        let rsrc_right = orientation.reface(src_right);
        // Now match up the faces
        let x_map = if rsrc_right == face_right {
            AxisMap::PosX
        } else if rsrc_right == face_down {
            AxisMap::PosY
        } else if rsrc_right == face_left {
            AxisMap::NegX
        } else {
            AxisMap::NegY
        };
        let y_map = if rsrc_up == face_up {
            AxisMap::PosY
        } else if rsrc_up == face_right {
            AxisMap::NegX
        } else if rsrc_up == face_down {
            AxisMap::NegY
        } else {
            AxisMap::PosX
        };
        CoordMap {
            x: x_map,
            y: y_map
        }
    }

    fn direction_coord(direction: Direction) -> (i8, i8, i8) {
        match direction {
            Direction::NegX => (-1, 0, 0),
            Direction::NegY => (0, -1, 0),
            Direction::NegZ => (0, 0, -1),
            Direction::PosX => (1, 0, 0),
            Direction::PosY => (0, 1, 0),
            Direction::PosZ => (0, 0, 1),
        }
    }

    fn coord_direction(coord: (i8, i8, i8)) -> Option<Direction> {
        match coord {
            (-1, 0, 0) => Some(Direction::NegX),
            (0, -1, 0) => Some(Direction::NegY),
            (0, 0, -1) => Some(Direction::NegZ),
            (1, 0, 0) => Some(Direction::PosX),
            (0, 1, 0) => Some(Direction::PosY),
            (0, 0, 1) => Some(Direction::PosZ),
            _ => None,
        }
    }

    #[test]
    fn rotation_tables_are_complete_unique_and_invertible() {
        let mut seen = HashSet::new();

        for rotation in all_rotations() {
            assert_eq!(rotation.left().invert(), rotation.right(), "{rotation}");
            assert_eq!(rotation.down().invert(), rotation.up(), "{rotation}");
            assert_eq!(rotation.forward().invert(), rotation.backward(), "{rotation}");

            let signature = Direction::iter()
                .map(|face| rotation.reface(face) as u8)
                .collect::<Vec<_>>();
            assert!(seen.insert(signature), "duplicate rotation mapping for {rotation}");

            for face in Direction::iter() {
                let rotated_coord = rotation.rotate(direction_coord(face));
                let rotated_face = coord_direction(rotated_coord)
                    .expect("rotating a face normal should produce another face normal");
                assert_eq!(rotated_face, rotation.reface(face), "{rotation} {face}");
                assert_eq!(rotation.source_face(rotation.reface(face)), face, "{rotation} {face}");
                assert_eq!(rotation.reface(rotation.source_face(face)), face, "{rotation} {face}");
            }
        }

        assert_eq!(seen.len(), 24);
    }

    #[test]
    fn from_up_and_forward_matches_rotation_set() {
        for up in Direction::iter() {
            for forward in Direction::iter() {
                let expected = all_rotations()
                    .find(|rotation| rotation.up() == up && rotation.forward() == forward);
                assert_eq!(
                    Rotation::from_up_and_forward(up, forward),
                    expected,
                    "up={up} forward={forward}",
                );
            }
        }
    }

    #[test]
    fn orientation_pack_reface_transform_and_inverse_are_consistent() {
        for orientation in all_orientations() {
            assert_eq!(Orientation::unpack(orientation.pack()), orientation, "{orientation}");

            let inverse = orientation.invert();
            assert_eq!(orientation.reorient(inverse), Orientation::UNORIENTED, "{orientation}");
            assert_eq!(orientation.deorient(orientation), Orientation::UNORIENTED, "{orientation}");

            for face in Direction::iter() {
                let transformed_coord = orientation.transform(direction_coord(face));
                let transformed_face = coord_direction(transformed_coord)
                    .expect("orienting a face normal should produce another face normal");
                assert_eq!(transformed_face, orientation.reface(face), "{orientation} {face}");
                assert_eq!(orientation.source_face(orientation.reface(face)), face, "{orientation} {face}");
                assert_eq!(orientation.reface(orientation.source_face(face)), face, "{orientation} {face}");
            }
        }
    }

    #[test]
    fn orientation_composition_round_trips() {
        for orient1 in all_orientations() {
            for orient2 in all_orientations() {
                assert_eq!(orient1.reorient(orient2).deorient(orient2), orient1);
                assert_eq!(orient1.deorient(orient2).reorient(orient2), orient1);
            }
        }
    }

    #[test]
    fn face_coord_tables_match_algorithmic_reference() {
        let sample_coords = [
            (-8, -8),
            (-5, 3),
            (-1, -2),
            (0, 0),
            (4, -7),
            (7, 7),
        ];

        for orientation in all_orientations() {
            for face in Direction::iter() {
                let expected_map = map_face_coord_naive(orientation, face);
                let expected_source = source_face_coord_naive(orientation, face);

                for coord in sample_coords {
                    assert_eq!(
                        orientation.map_face_coord(face, coord),
                        expected_map.map(coord),
                        "{orientation} {face} map {coord:?}",
                    );
                    assert_eq!(
                        orientation.source_face_coord(face, coord),
                        expected_source.map(coord),
                        "{orientation} {face} source {coord:?}",
                    );

                    let target_coord = orientation.map_face_coord(face, coord);
                    assert_eq!(
                        orientation.source_face_coord(face, target_coord),
                        coord,
                        "{orientation} {face} source(map(coord))",
                    );

                    let source_coord = orientation.source_face_coord(face, coord);
                    assert_eq!(
                        orientation.map_face_coord(face, source_coord),
                        coord,
                        "{orientation} {face} map(source(coord))",
                    );
                }
            }
        }
    }

    #[test]
    fn deorient_test() {
        Direction::iter().for_each(|up1| {
            (0..4).for_each(|angle1| {
                (0..8).map(|i| Orientation::new(Rotation::new(up1, angle1), Flip(i))).for_each(|orient1| {
                    Direction::iter().for_each(|up2| {
                        (0..4).for_each(|angle2| {
                            (0..8).map(|i| Orientation::new(Rotation::new(up2, angle2), Flip(i))).for_each(|orient2| {
                                let reorient = orient1.reorient(orient2);
                                let deorient = reorient.deorient(orient2);
                                assert_eq!(deorient, orient1);
                            });
                        });
                    });
                })
            })
        });
    }

    // This is used to generate the table in maptable.rs.
    #[test]
    #[ignore]
    fn map_coord_gencode() {
        const fn map_axismap(a: AxisMap) -> &'static str {
            match a {
                AxisMap::PosX => "x",
                AxisMap::PosY => "y",
                AxisMap::NegX => "-x",
                AxisMap::NegY => "-y",
            }
        }
        let output = {
            use std::fmt::Write;
            let mut output = String::new();
            let mut count = 0usize;
            for flipi in 0..8 { // flip
                for roti in 0..24 { // rotation
                    Direction::iter_index_order().for_each(|face| {
                        count += 1;
                        let map = map_face_coord_naive(Orientation::new(Rotation(roti as u8), Flip(flipi as u8)), face);
                        writeln!(output, "CoordMap::new(AxisMap::{:?}, AxisMap::{:?}),", map.x, map.y);
                    });
                }
            }
            output
        };
        use std::io::{Write, BufWriter};
        use std::fs::File;
        std::fs::create_dir_all("ignore").ok();
        let mut writer = BufWriter::new(File::create("ignore/map_coord_table.rs").expect("Failed to open file"));
        writer.write_all(output.as_bytes());
        println!("Wrote the output to file at ./ignore/map_coord_table.rs");
    }
    #[test]
    #[ignore]
    fn source_coord_gencode() {
        const fn map_axismap(a: AxisMap) -> &'static str {
            match a {
                AxisMap::PosX => "x",
                AxisMap::PosY => "y",
                AxisMap::NegX => "-x",
                AxisMap::NegY => "-y",
            }
        }
        let output = {
            use std::fmt::Write;
            let mut output = String::new();
            let mut count = 0usize;
            for flipi in 0..8 { // flip
                for roti in 0..24 { // rotation
                    Direction::iter_index_order().for_each(|face| {
                        count += 1;
                        let map = source_face_coord_naive(Orientation::new(Rotation(roti as u8), Flip(flipi as u8)), face);
                        writeln!(output, "CoordMap::new(AxisMap::{:?}, AxisMap::{:?}),", map.x, map.y);
                    });
                }
            }
            output
        };
        use std::io::{Write, BufWriter};
        use std::fs::File;
        std::fs::create_dir_all("ignore").ok();
        let mut writer = BufWriter::new(File::create("ignore/source_face_coord_table.rs").expect("Failed to open file"));
        writer.write_all(output.as_bytes());
        println!("Wrote the output to file at ./ignore/source_face_coord_table.rs");
    }

}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Orientation({},{})", self.flip, self.rotation)
    }
}
