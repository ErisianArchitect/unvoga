#![allow(unused)]
use bevy::math::{vec3, Vec3};
use bytemuck::NoUninit;
use super::{
    flip::*,
    orientation::*,
    rotation::*,
};

use crate::core::voxel::direction::Direction;

pub const fn pack_flip_and_rotation(flip: Flip, rotation: Rotation) -> u8 {
    flip.0 | rotation.0 << 3
}

pub const fn unpack_flip_and_rotation(packed: u8) -> (Flip, Rotation) {
    let flip = packed & 0b111;
    let rotation = packed >> 3;
    (Flip(flip), Rotation(rotation))
}

pub const fn rotate_face_coord(angle: u8, x: usize, y: usize, size: usize) -> (usize, usize) {
    match angle & 0b11 {
        0 => (x, y),
        1 => (size - y, x),
        2 => (size - x, size - y),
        3 => (y, size - x),
        _ => unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use bevy::{asset::io::memory::Dir, math::vec3};

    use crate::core::voxel::direction::Direction;

    use super::*;

    #[test]
    fn orientation_source_face_test() {
        use Direction::*;
        let orient = Orientation::new(Rotation::new(NegX, 1), Flip::X | Flip::Y);
        let neg_x = orient.source_face(NegX);
        let neg_y = orient.source_face(NegY);
        let neg_z = orient.source_face(NegZ);
        let pos_x = orient.source_face(PosX);
        let pos_y = orient.source_face(PosY);
        let pos_z = orient.source_face(PosZ);
        assert_eq!(neg_x, NegY);
        assert_eq!(neg_y, NegX);
        assert_eq!(neg_z, PosZ);
        assert_eq!(pos_x, PosY);
        assert_eq!(pos_y, PosX);
        assert_eq!(pos_z, NegZ);
    }

    #[test]
    fn pack_test() {
        let flip = Flip::Z | Flip::Y;
        let rotation = Rotation::new(Direction::PosX, 1);
        let packed = pack_flip_and_rotation(flip, rotation);
        let (uflip, urot) = unpack_flip_and_rotation(packed);
        assert_eq!((flip, rotation), (uflip, urot));
    }

    #[test]
    fn up_and_fwd_test() {
        Direction::iter().for_each(|up| Direction::iter().for_each(|forward| {
            let rotation = Rotation::from_up_and_forward(up, forward);
            if let Some(rotation) = rotation {
                assert_eq!(up, rotation.up());
                assert_eq!(forward, rotation.forward());
            } else {
                if forward != up && forward.invert() != up {
                    panic!("None when Some expected");
                }
            }
        }));
    }

    #[test]
    fn face_rotation_test() {
        let rots = [
            (0, Direction::NegZ),
            (1, Direction::PosX),
            (2, Direction::PosZ),
            (3, Direction::NegX)
        ];
        use Direction::*;
        Direction::iter().for_each(|up| (0..4).for_each(|angle| {
            let rot = Rotation::new(up, angle);
            assert_eq!(rot.forward(), rot.reface(NegZ));
            assert_eq!(rot.right(), rot.reface(PosX));
            assert_eq!(rot.backward(), rot.reface(PosZ));
            assert_eq!(rot.left(), rot.reface(NegX));
        }));

    }
    use Direction::*;
    fn face_up(face: Direction) -> Direction {
        match face {
            NegX => PosY,
            NegY => PosZ,
            NegZ => PosY,
            PosX => PosY,
            PosY => NegZ,
            PosZ => PosY,
        }
    }
    fn face_down(face: Direction) -> Direction {
        match face {
            NegX => NegY,
            NegY => NegZ,
            NegZ => NegY,
            PosX => NegY,
            PosY => PosZ,
            PosZ => NegY,
        }
    }
    fn face_left(face: Direction) -> Direction {
        match face {
            NegX => NegZ,
            NegY => NegX,
            NegZ => PosX,
            PosX => PosZ,
            PosY => NegX,
            PosZ => NegX,
        }
    }
    fn face_right(face: Direction) -> Direction {
        match face {
            NegX => PosZ,
            NegY => PosX,
            NegZ => NegX,
            PosX => NegZ,
            PosY => PosX,
            PosZ => PosX,
        }
    }
    fn write_file<P: AsRef<std::path::Path>, S: AsRef<str>>(path: P, content: S) -> Result<(), std::io::Error> {
        use std::fs::File;
        use std::io::{Write, BufWriter};
        let mut out = BufWriter::new(File::create(path)?);
        write!(out, "{}", content.as_ref())
    }
    #[test]
    fn src_test() -> Result<(), std::io::Error> {
        fn face_rotation(rot: Rotation, face: Direction) -> u8 {
            let source_face = rot.source_face(face);
            let up = face_up(face);
            let faces = [
                (0, rot.reface(face_up(source_face))),
                (3, rot.reface(face_right(source_face))),
                (2, rot.reface(face_down(source_face))),
                (1, rot.reface(face_left(source_face)))
            ];
            faces.into_iter().find_map(move |(angle, face)| {
                if up == face {
                    Some(angle)
                } else {
                    None
                }
            }).unwrap()
        }
        let rot = Rotation::new(NegY, 1);
        let face = NegZ;
        let source_face = rot.source_face(face);
        println!("Source Face: {source_face:?}");
        let src_left = face_left(source_face);
        println!("Face Left: {src_left:?}");
        let my_fwd = face_up(face);
        let faces = [
            (0, rot.reface(face_up(source_face))),
            (3, rot.reface(face_right(source_face))),
            (2, rot.reface(face_down(source_face))),
            (1, rot.reface(face_left(source_face)))
        ];
        // faces.into_iter().for_each(|(angle, face)| {
        //     if my_fwd == face {
        //         println!("Angle: {angle}");
        //     }
        // });
        let mut fr1 = String::new();
        let mut fr2 = String::new();
        use std::fmt::Write;
        writeln!(fr1, "match (self.angle(), self.up(), face) {{");
        writeln!(fr2, "match self.angle() {{");
        (0..4).for_each(|angle| {
            writeln!(fr2, "    {angle} => match self.up() {{");
            Direction::iter().for_each(|up| {
                let rot = Rotation::new(up, angle);
                writeln!(fr2, "        {up:?} => match face {{");
                Direction::iter().for_each(|face| {
                    let face_rot = face_rotation(rot, face);
                    writeln!(fr2, "            {face:?} => {face_rot},");
                    writeln!(fr1, "    ({angle}, {up:?}, {face:?}) => {face_rot},");
                    println!("({angle},{up:?},{face:?}) => {face_rot}");
                });
                writeln!(fr2, "        }}");
            });
            writeln!(fr2, "    }}");
        });
        writeln!(fr2, "    _ => unreachable!()");
        writeln!(fr1, "    _ => unreachable!()");
        write!(fr1, "}}");
        write!(fr2, "}}");
        write_file("ignore/face_rotation1.rs", fr1)?;
        write_file("ignore/face_rotation2.rs", fr2)?;
        println!("Files written!");
        Ok(())
        // Direction::iter().for_each(|up|(0..4).for_each(|rot| {
        //     let rot = Rotation::new(up, rot);
        //     Direction::iter().for_each(|dest| {
        //         let src = rot.source_face(dest);
        //         let rot_src = rot.reface(src);
        //         assert_eq!(rot_src, dest);
        //     });
        // }));
    }

    #[test]
    fn flip_test() {
        let flip = Flip::X | Flip::Y;
        assert_eq!(Flip(3), flip);
        assert_eq!(Flip::X, flip - Flip::Y);
    }

    #[test]
    fn rotation_test() {
        Direction::iter().for_each(|dir| {
            assert_eq!(Rotation::new(Direction::PosY, 0).source_face(dir), dir);
        });
        Direction::iter().for_each(|dir| (0..4).for_each(|rot| {
            let rot = Rotation::new(dir, rot);
            println!("      Up: {:?}", rot.up());
            println!(" Forward: {:?}", rot.forward());
            println!("Rotation: {}", rot.angle());
        }));
    }

    #[test]
    fn translate_test() {
        let offset = vec3(1.0, 1.0, 0.0);
        let rot = Rotation::new(Direction::NegZ, 1);
        let trans = rot.rotate(offset);
        println!("{trans}");
    }

    #[test]
    fn map_test() {
        let dir = Direction::PosY;
        let rot = Rotation::new(Direction::PosZ, 0);
        let find = rot.source_face(Direction::PosY);
        println!("{find:?}");
    }

    #[test]
    fn bootstrap_gen() -> std::io::Result<()> {
        use std::fs::File;
        use std::io::BufWriter;
        use std::io::Write;
        // use std::fmt::Write;
        let mut file = File::create("./codegen.rs")?;
        let mut file = BufWriter::new(file);
        // let mut file = String::new();
        writeln!(file, "use Direction::*;")?;
        writeln!(file, "match self.up() {{")?;
        let i = "    ";
        Direction::iter().try_for_each(|dir| {
            writeln!(file, "{i}{dir:?} => match self.rotation() {{")?;
            (0..4).try_for_each(|rot| {
                writeln!(file, "{i}{i}{rot} => match destination {{")?;
                Direction::iter().try_for_each(|dest| {
                    writeln!(file, "{i}{i}{i}{dest:?} => {:?},", Rotation::new(dir, rot).source_face(dest))
                });
                writeln!(file, "{i}{i}}}")
            });
            writeln!(file, "{i}{i}_ => unreachable!()\n    }}")
        });
        writeln!(file, "}}")?;
        println!("Code written to file.");
        Ok(())
    }

    #[test]
    fn cycle_test() {
        let mut rot = Rotation::new(Direction::PosY, 0);
        println!("{rot}");
        for i in 0..24 {
            rot = rot.cycle(1);
            println!("{rot}");
        }
    }
}