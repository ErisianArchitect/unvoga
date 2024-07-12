use bevy::math::{vec3, Vec3};

use crate::core::voxel::direction::Direction;


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rotation(u8);

impl Rotation {
    pub const fn new(up: Direction, angle: i32) -> Self {
        let up = up as u8;
        let rotation = angle.rem_euclid(4) as u8;
        Self(rotation | up << 2)
    }

    pub const fn from_up_and_forward(up: Direction, forward: Direction) -> Option<Rotation> {
        Some(Rotation::new(up, match up {
            Direction::NegX => match forward {
                Direction::NegX => return None,
                Direction::NegY => 2,
                Direction::NegZ => 3,
                Direction::PosX => return None,
                Direction::PosY => 0,
                Direction::PosZ => 1
            },
            Direction::NegY => match forward {
                Direction::NegX => 3,
                Direction::NegY => return None,
                Direction::NegZ => 2,
                Direction::PosX => 1,
                Direction::PosY => return None,
                Direction::PosZ => 0
            },
            Direction::NegZ => match forward {
                Direction::NegX => 1,
                Direction::NegY => 2,
                Direction::NegZ => return None,
                Direction::PosX => 3,
                Direction::PosY => 0,
                Direction::PosZ => return None
            },
            Direction::PosX => match forward {
                Direction::NegX => return None,
                Direction::NegY => 2,
                Direction::NegZ => 1,
                Direction::PosX => return None,
                Direction::PosY => 0,
                Direction::PosZ => 3
            },
            Direction::PosY => match forward {
                Direction::NegX => 3,
                Direction::NegY => return None,
                Direction::NegZ => 0,
                Direction::PosX => 1,
                Direction::PosY => return None,
                Direction::PosZ => 2
            },
            Direction::PosZ => match forward {
                Direction::NegX => 3,
                Direction::NegY => 2,
                Direction::NegZ => return None,
                Direction::PosX => 1,
                Direction::PosY => 0,
                Direction::PosZ => return None
            },
        }))
    }

    pub const fn angle(self) -> i32 {
        (self.0 & 0b11) as i32
    }

    pub const fn up(self) -> Direction {
        let up = self.0 >> 2 & 0b111;
        match up {
            4 => Direction::NegX,
            3 => Direction::NegY,
            5 => Direction::NegZ,
            1 => Direction::PosX,
            0 => Direction::PosY,
            2 => Direction::PosZ,
            _ => unreachable!(),
        }
    }

    pub const fn down(self) -> Direction {
        let up = self.0 >> 2 & 0b111;
        match up {
            4 => Direction::PosX,
            3 => Direction::PosY,
            5 => Direction::PosZ,
            1 => Direction::NegX,
            0 => Direction::NegY,
            2 => Direction::NegZ,
            _ => unreachable!(),
        }
    }

    pub const fn forward(self) -> Direction {
        use Direction::*;
        match self.up() {
            Direction::NegX => match self.angle() {
                0 => PosY,
                1 => PosZ,
                2 => NegY,
                3 => NegZ,
                _ => unreachable!()
            }
            Direction::NegY => match self.angle() {
                0 => PosZ,
                1 => PosX,
                2 => NegZ,
                3 => NegX,
                _ => unreachable!()
            }
            Direction::NegZ => match self.angle() {
                0 => PosY,
                1 => NegX,
                2 => NegY,
                3 => PosX,
                _ => unreachable!()
            }
            Direction::PosX => match self.angle() {
                0 => PosY,
                1 => NegZ,
                2 => NegY,
                3 => PosZ,
                _ => unreachable!()
            }
            Direction::PosY => match self.angle() {
                0 => NegZ,
                1 => PosX,
                2 => PosZ,
                3 => NegX,
                _ => unreachable!()
            }
            Direction::PosZ => match self.angle() {
                0 => PosY,
                1 => PosX,
                2 => NegY,
                3 => NegX,
                _ => unreachable!()
            }
        }
    }

    pub const fn backward(self) -> Direction {
        // self.forward().invert()
        use Direction::*;
        match self.up() {
            NegX => match self.angle() {
                0 => NegY,
                1 => NegZ,
                2 => PosY,
                3 => PosZ,
                _ => unreachable!()
            }
            NegY => match self.angle() {
                0 => NegZ,
                1 => NegX,
                2 => PosZ,
                3 => PosX,
                _ => unreachable!()
            }
            NegZ => match self.angle() {
                0 => NegY,
                1 => PosX,
                2 => PosY,
                3 => NegX,
                _ => unreachable!()
            }
            PosX => match self.angle() {
                0 => NegY,
                1 => PosZ,
                2 => PosY,
                3 => NegZ,
                _ => unreachable!()
            }
            PosY => match self.angle() {
                0 => PosZ,
                1 => NegX,
                2 => NegZ,
                3 => PosX,
                _ => unreachable!()
            }
            PosZ => match self.angle() {
                0 => NegY,
                1 => NegX,
                2 => PosY,
                3 => PosX,
                _ => unreachable!()
            }
        }
    }

    pub const fn left(self) -> Direction {
        use Direction::*;
        match self.up() {
            NegX => match self.angle() {
                0 => NegZ,
                1 => PosY,
                2 => PosZ,
                3 => NegY,
                _ => unreachable!()
            }
            NegY => match self.angle() {
                0 => NegX,
                1 => PosZ,
                2 => PosX,
                3 => NegZ,
                _ => unreachable!()
            }
            NegZ => match self.angle() {
                0 => PosX,
                1 => PosY,
                2 => NegX,
                3 => NegY,
                _ => unreachable!()
            }
            PosX => match self.angle() {
                0 => PosZ,
                1 => PosY,
                2 => NegZ,
                3 => NegY,
                _ => unreachable!()
            }
            PosY => match self.angle() {
                0 => NegX,
                1 => NegZ,
                2 => PosX,
                3 => PosZ,
                _ => unreachable!()
            }
            PosZ => match self.angle() {
                0 => NegX,
                1 => PosY,
                2 => PosX,
                3 => NegY,
                _ => unreachable!()
            }
        }
    }

    pub const fn right(self) -> Direction {
        use Direction::*;
        match self.up() {
            NegX => match self.angle() {
                0 => PosZ,
                1 => NegY,
                2 => NegZ,
                3 => PosY,
                _ => unreachable!()
            }
            NegY => match self.angle() {
                0 => PosX,
                1 => NegZ,
                2 => NegX,
                3 => PosZ,
                _ => unreachable!()
            }
            NegZ => match self.angle() {
                0 => NegX,
                1 => NegY,
                2 => PosX,
                3 => PosY,
                _ => unreachable!()
            }
            PosX => match self.angle() {
                0 => NegZ,
                1 => NegY,
                2 => PosZ,
                3 => PosY,
                _ => unreachable!()
            }
            PosY => match self.angle() {
                0 => PosX,
                1 => PosZ,
                2 => NegX,
                3 => NegZ,
                _ => unreachable!()
            }
            PosZ => match self.angle() {
                0 => PosX,
                1 => NegY,
                2 => NegX,
                3 => PosY,
                _ => unreachable!()
            }
        }
    }

    /// Rotates `coord`.
    pub fn rotate(self, coord: Vec3) -> Vec3 {
        match self.up() {
            Direction::NegX => match self.angle() {
                0 => vec3(-coord.y, -coord.z, coord.x),
                1 => vec3(-coord.y, -coord.x, -coord.z),
                2 => vec3(-coord.y, coord.z, -coord.x),
                3 => vec3(-coord.y, coord.x, coord.z),
                _ => unreachable!()
            },
            Direction::NegY => match self.angle() {
                0 => vec3(coord.x, -coord.y, -coord.z),
                1 => vec3(-coord.z, -coord.y, -coord.x),
                2 => vec3(-coord.x, -coord.y, coord.z),
                3 => vec3(coord.z, -coord.y, coord.x),
                _ => unreachable!()
            },
            Direction::NegZ => match self.angle() {
                0 => vec3(-coord.x, -coord.z, -coord.y),
                1 => vec3(coord.z, -coord.x, -coord.y),
                2 => vec3(coord.x, coord.z, -coord.y),
                3 => vec3(-coord.z, coord.x, -coord.y),
                _ => unreachable!()
            },
            Direction::PosX => match self.angle() {
                0 => vec3(coord.y, -coord.z, -coord.x),
                1 => vec3(coord.y, -coord.x, coord.z),
                2 => vec3(coord.y, coord.z, coord.x),
                3 => vec3(coord.y, coord.x, -coord.z),
                _ => unreachable!()
            },
            Direction::PosY => match self.angle() {
                0 => coord, // Default rotation, no change.
                1 => vec3(-coord.z, coord.y, coord.x),
                2 => vec3(-coord.x, coord.y, -coord.z),
                3 => vec3(coord.z, coord.y, -coord.x),
                _ => unreachable!()
            },
            Direction::PosZ => match self.angle() {
                0 => vec3(coord.x, -coord.z, coord.y),
                1 => vec3(-coord.z, -coord.x, coord.y),
                2 => vec3(-coord.x, coord.z, coord.y),
                3 => vec3(coord.z, coord.x, coord.y),
                _ => unreachable!()
            },
        }
    }

    /// Rotates direction.
    pub fn reface(self, direction: Direction) -> Direction {
        match direction {
            Direction::NegX => self.left(),
            Direction::NegY => self.down(),
            Direction::NegZ => self.forward(),
            Direction::PosX => self.right(),
            Direction::PosY => self.up(),
            Direction::PosZ => self.backward(),
        }
    }

    /// Tells which [Direction] rotated to `destination`.
    pub fn source_face(self, destination: Direction) -> Direction {
        // This code was bootstrap generated. I wrote a naive solution,
        // then generated this code with the naive solution.
        // Besides maybe if you rearrange the order of matching,
        // this should be theoretically the optimal solution.
        use Direction::*;
        match self.up() {
            NegX => match self.angle() {
                0 => match destination {
                    NegX => PosY,
                    NegY => PosZ,
                    NegZ => NegX,
                    PosX => NegY,
                    PosY => NegZ,
                    PosZ => PosX,
                }
                1 => match destination {
                    NegX => PosY,
                    NegY => PosX,
                    NegZ => PosZ,
                    PosX => NegY,
                    PosY => NegX,
                    PosZ => NegZ,
                }
                2 => match destination {
                    NegX => PosY,
                    NegY => NegZ,
                    NegZ => PosX,
                    PosX => NegY,
                    PosY => PosZ,
                    PosZ => NegX,
                }
                3 => match destination {
                    NegX => PosY,
                    NegY => NegX,
                    NegZ => NegZ,
                    PosX => NegY,
                    PosY => PosX,
                    PosZ => PosZ,
                }
                _ => unreachable!()
            }
            NegY => match self.angle() {
                0 => match destination {
                    NegX => NegX,
                    NegY => PosY,
                    NegZ => PosZ,
                    PosX => PosX,
                    PosY => NegY,
                    PosZ => NegZ,
                }
                1 => match destination {
                    NegX => PosZ,
                    NegY => PosY,
                    NegZ => PosX,
                    PosX => NegZ,
                    PosY => NegY,
                    PosZ => NegX,
                }
                2 => match destination {
                    NegX => PosX,
                    NegY => PosY,
                    NegZ => NegZ,
                    PosX => NegX,
                    PosY => NegY,
                    PosZ => PosZ,
                }
                3 => match destination {
                    NegX => NegZ,
                    NegY => PosY,
                    NegZ => NegX,
                    PosX => PosZ,
                    PosY => NegY,
                    PosZ => PosX,
                }
                _ => unreachable!()
            }
            NegZ => match self.angle() {
                0 => match destination {
                    NegX => PosX,
                    NegY => PosZ,
                    NegZ => PosY,
                    PosX => NegX,
                    PosY => NegZ,
                    PosZ => NegY,
                }
                1 => match destination {
                    NegX => NegZ,
                    NegY => PosX,
                    NegZ => PosY,
                    PosX => PosZ,
                    PosY => NegX,
                    PosZ => NegY,
                }
                2 => match destination {
                    NegX => NegX,
                    NegY => NegZ,
                    NegZ => PosY,
                    PosX => PosX,
                    PosY => PosZ,
                    PosZ => NegY,
                }
                3 => match destination {
                    NegX => PosZ,
                    NegY => NegX,
                    NegZ => PosY,
                    PosX => NegZ,
                    PosY => PosX,
                    PosZ => NegY,
                }
                _ => unreachable!()
            }
            PosX => match self.angle() {
                0 => match destination {
                    NegX => NegY,
                    NegY => PosZ,
                    NegZ => PosX,
                    PosX => PosY,
                    PosY => NegZ,
                    PosZ => NegX,
                }
                1 => match destination {
                    NegX => NegY,
                    NegY => PosX,
                    NegZ => NegZ,
                    PosX => PosY,
                    PosY => NegX,
                    PosZ => PosZ,
                }
                2 => match destination {
                    NegX => NegY,
                    NegY => NegZ,
                    NegZ => NegX,
                    PosX => PosY,
                    PosY => PosZ,
                    PosZ => PosX,
                }
                3 => match destination {
                    NegX => NegY,
                    NegY => NegX,
                    NegZ => PosZ,
                    PosX => PosY,
                    PosY => PosX,
                    PosZ => NegZ,
                }
                _ => unreachable!()
            }
            PosY => match self.angle() {
                0 => match destination {
                    NegX => NegX,
                    NegY => NegY,
                    NegZ => NegZ,
                    PosX => PosX,
                    PosY => PosY,
                    PosZ => PosZ,
                }
                1 => match destination {
                    NegX => PosZ,
                    NegY => NegY,
                    NegZ => NegX,
                    PosX => NegZ,
                    PosY => PosY,
                    PosZ => PosX,
                }
                2 => match destination {
                    NegX => PosX,
                    NegY => NegY,
                    NegZ => PosZ,
                    PosX => NegX,
                    PosY => PosY,
                    PosZ => NegZ,
                }
                3 => match destination {
                    NegX => NegZ,
                    NegY => NegY,
                    NegZ => PosX,
                    PosX => PosZ,
                    PosY => PosY,
                    PosZ => NegX,
                }
                _ => unreachable!()
            }
            PosZ => match self.angle() {
                0 => match destination {
                    NegX => NegX,
                    NegY => PosZ,
                    NegZ => NegY,
                    PosX => PosX,
                    PosY => NegZ,
                    PosZ => PosY,
                }
                1 => match destination {
                    NegX => PosZ,
                    NegY => PosX,
                    NegZ => NegY,
                    PosX => NegZ,
                    PosY => NegX,
                    PosZ => PosY,
                }
                2 => match destination {
                    NegX => PosX,
                    NegY => NegZ,
                    NegZ => NegY,
                    PosX => NegX,
                    PosY => PosZ,
                    PosZ => PosY,
                }
                3 => match destination {
                    NegX => NegZ,
                    NegY => NegX,
                    NegZ => NegY,
                    PosX => PosZ,
                    PosY => PosX,
                    PosZ => PosY,
                }
                _ => unreachable!()
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Flip(u8);

pub fn pack_flip_and_rotation(flip: Flip, rotation: Rotation) -> u8 {
    flip.0 | rotation.0 << 3
}

pub fn unpack_flip_and_rotation(packed: u8) -> (Flip, Rotation) {
    let flip = packed & 0b111;
    let rotation = packed >> 3;
    (Flip(flip), Rotation(rotation))
}

impl Flip {
    pub const X: Flip = Flip(1);
    pub const Y: Flip = Flip(2);
    pub const Z: Flip = Flip(4);
    pub const ALL: Flip = Flip(7);
    pub const NONE: Flip = Flip(0);

    pub fn x(self) -> bool {
        self & Flip::X == Flip::X
    }

    pub fn y(self) -> bool {
        self & Flip::Y == Flip::Y
    }

    pub fn z(self) -> bool {
        self & Flip::Z == Flip::Z
    }
}

impl std::ops::BitOr<Flip> for Flip {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Flip) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign<Flip> for Flip {
    fn bitor_assign(&mut self, rhs: Flip) {
        *self = *self | rhs;
    }
}

impl std::ops::BitAndAssign<Flip> for Flip {
    fn bitand_assign(&mut self, rhs: Flip) {
        *self = *self & rhs;
    }
}

impl std::ops::Add<Flip> for Flip {
    type Output = Flip;
    #[inline]
    fn add(self, rhs: Flip) -> Self::Output {
        self | rhs
    }
}

impl std::ops::Sub<Flip> for Flip {
    type Output = Flip;
    #[inline]
    fn sub(self, rhs: Flip) -> Self::Output {
        self & !rhs
    }
}

impl std::ops::BitAnd<Flip> for Flip {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Flip) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::Not for Flip {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0 & 0b111)
    }
}


#[cfg(test)]
mod tests {
    use bevy::math::vec3;

    use crate::core::{math::coordmap::Flip, voxel::direction::Direction};

    use super::{pack_flip_and_rotation, unpack_flip_and_rotation, Rotation};

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
    fn src_test() {
        Direction::iter().for_each(|up|(0..4).for_each(|rot| {
            let rot = Rotation::new(up, rot);
            Direction::iter().for_each(|dest| {
                let src = rot.source_face(dest);
                let rot_src = rot.reface(src);
                assert_eq!(rot_src, dest);
            });
        }));
    }

    #[test]
    fn flip_test() {
        let flip = Flip::X | Flip::Y;
        assert_eq!(Flip(3), flip);
        assert_eq!(Flip::X, flip - Flip::Y);
    }

    #[test]
    fn rotation_test() {
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
}