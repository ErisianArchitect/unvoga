#![allow(unused)]
use bevy::math::{vec3, Vec3};
use bytemuck::NoUninit;

use crate::prelude::*;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, NoUninit)]
pub struct Rotation(pub u8);

impl Rotation {
    pub const UNROTATED: Rotation = Rotation::new(Direction::PosY, 0);
    pub const fn new(up: Direction, angle: i32) -> Self {
        let up = up as u8;
        let angle = angle.rem_euclid(4) as u8;
        Self(angle | up << 2)
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

    /// Cycle through rotations (24 in total).
    #[must_use]
    pub const fn cycle(self, offset: i32) -> Rotation {
        let index = self.0 as i32;
        let new_index = (index as i64 + offset as i64).rem_euclid(24) as u8;
        Rotation(new_index)
    }

    pub const fn angle(self) -> i32 {
        (self.0 & 0b11) as i32
    }

    pub const fn up(self) -> Direction {
        let up = self.0 >> 2;
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
        let up = self.0 >> 2;
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

    pub const fn reorient(self, rotation: Self) -> Self {
        let up = self.up();
        let fwd = self.forward();
        let rot_up = rotation.reface(up);
        let rot_fwd = rotation.reface(fwd);
        let Some(rot) = Self::from_up_and_forward(rot_up, rot_fwd) else {
            unreachable!()
        };
        rot
    }

    pub const fn deorient(self, rotation: Self) -> Self {
        let up = self.up();
        let fwd = self.forward();
        let rot_up = rotation.source_face(up);
        let rot_fwd = rotation.source_face(fwd);
        let Some(rot) = Self::from_up_and_forward(rot_up, rot_fwd) else {
            unreachable!()
        };
        rot
    }

    pub const fn invert(self) -> Self {
        Self::UNROTATED.deorient(self)
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
    pub fn rotate<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T, T)> + From<(T, T, T)>>(self, coord: C) -> C {
        let (x, y, z): (T, T, T) = coord.into();
        C::from(match self.up() {
            Direction::NegX => match self.angle() {
                0 => (-y, -z, x),
                1 => (-y, -x, -z),
                2 => (-y, z, -x),
                3 => (-y, x, z),
                _ => unreachable!()
            },
            Direction::NegY => match self.angle() {
                0 => (x, -y, -z),
                1 => (-z, -y, -x),
                2 => (-x, -y, z),
                3 => (z, -y, x),
                _ => unreachable!()
            },
            Direction::NegZ => match self.angle() {
                0 => (-x, -z, -y),
                1 => (z, -x, -y),
                2 => (x, z, -y),
                3 => (-z, x, -y),
                _ => unreachable!()
            },
            Direction::PosX => match self.angle() {
                0 => (y, -z, -x),
                1 => (y, -x, z),
                2 => (y, z, x),
                3 => (y, x, -z),
                _ => unreachable!()
            },
            Direction::PosY => match self.angle() {
                0 => (x, y, z), // Default rotation, no change.
                1 => (-z, y, x),
                2 => (-x, y, -z),
                3 => (z, y, -x),
                _ => unreachable!()
            },
            Direction::PosZ => match self.angle() {
                0 => (x, -z, y),
                1 => (-z, -x, y),
                2 => (-x, z, y),
                3 => (z, x, y),
                _ => unreachable!()
            },
        })
    }

    /// Rotates direction.
    pub const fn reface(self, direction: Direction) -> Direction {
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
    pub const fn source_face(self, destination: Direction) -> Direction {
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

    /// Gets the angle of the source face. 
    pub fn face_angle(self, face: Direction) -> u8 {
        use Direction::*;
        match (self.angle(), self.up(), face) {
            (0, NegX, NegX) => 0,
            (0, NegX, NegY) => 3,
            (0, NegX, NegZ) => 1,
            (0, NegX, PosX) => 2,
            (0, NegX, PosY) => 3,
            (0, NegX, PosZ) => 3,
            (0, NegY, NegX) => 2,
            (0, NegY, NegY) => 0,
            (0, NegY, NegZ) => 2,
            (0, NegY, PosX) => 2,
            (0, NegY, PosY) => 0,
            (0, NegY, PosZ) => 2,
            (0, NegZ, NegX) => 3,
            (0, NegZ, NegY) => 2,
            (0, NegZ, NegZ) => 0,
            (0, NegZ, PosX) => 1,
            (0, NegZ, PosY) => 0,
            (0, NegZ, PosZ) => 2,
            (0, PosX, NegX) => 2,
            (0, PosX, NegY) => 1,
            (0, PosX, NegZ) => 3,
            (0, PosX, PosX) => 0,
            (0, PosX, PosY) => 1,
            (0, PosX, PosZ) => 1,
            (0, PosY, NegX) => 0,
            (0, PosY, NegY) => 0,
            (0, PosY, NegZ) => 0,
            (0, PosY, PosX) => 0,
            (0, PosY, PosY) => 0,
            (0, PosY, PosZ) => 0,
            (0, PosZ, NegX) => 1,
            (0, PosZ, NegY) => 0,
            (0, PosZ, NegZ) => 2,
            (0, PosZ, PosX) => 3,
            (0, PosZ, PosY) => 2,
            (0, PosZ, PosZ) => 0,
            (1, NegX, NegX) => 1,
            (1, NegX, NegY) => 3,
            (1, NegX, NegZ) => 1,
            (1, NegX, PosX) => 1,
            (1, NegX, PosY) => 3,
            (1, NegX, PosZ) => 3,
            (1, NegY, NegX) => 2,
            (1, NegY, NegY) => 1,
            (1, NegY, NegZ) => 2,
            (1, NegY, PosX) => 2,
            (1, NegY, PosY) => 3,
            (1, NegY, PosZ) => 2,
            (1, NegZ, NegX) => 3,
            (1, NegZ, NegY) => 2,
            (1, NegZ, NegZ) => 1,
            (1, NegZ, PosX) => 1,
            (1, NegZ, PosY) => 0,
            (1, NegZ, PosZ) => 1,
            (1, PosX, NegX) => 1,
            (1, PosX, NegY) => 1,
            (1, PosX, NegZ) => 3,
            (1, PosX, PosX) => 1,
            (1, PosX, PosY) => 1,
            (1, PosX, PosZ) => 1,
            (1, PosY, NegX) => 0,
            (1, PosY, NegY) => 3,
            (1, PosY, NegZ) => 0,
            (1, PosY, PosX) => 0,
            (1, PosY, PosY) => 1,
            (1, PosY, PosZ) => 0,
            (1, PosZ, NegX) => 1,
            (1, PosZ, NegY) => 0,
            (1, PosZ, NegZ) => 1,
            (1, PosZ, PosX) => 3,
            (1, PosZ, PosY) => 2,
            (1, PosZ, PosZ) => 1,
            (2, NegX, NegX) => 2,
            (2, NegX, NegY) => 3,
            (2, NegX, NegZ) => 1,
            (2, NegX, PosX) => 0,
            (2, NegX, PosY) => 3,
            (2, NegX, PosZ) => 3,
            (2, NegY, NegX) => 2,
            (2, NegY, NegY) => 2,
            (2, NegY, NegZ) => 2,
            (2, NegY, PosX) => 2,
            (2, NegY, PosY) => 2,
            (2, NegY, PosZ) => 2,
            (2, NegZ, NegX) => 3,
            (2, NegZ, NegY) => 2,
            (2, NegZ, NegZ) => 2,
            (2, NegZ, PosX) => 1,
            (2, NegZ, PosY) => 0,
            (2, NegZ, PosZ) => 0,
            (2, PosX, NegX) => 0,
            (2, PosX, NegY) => 1,
            (2, PosX, NegZ) => 3,
            (2, PosX, PosX) => 2,
            (2, PosX, PosY) => 1,
            (2, PosX, PosZ) => 1,
            (2, PosY, NegX) => 0,
            (2, PosY, NegY) => 2,
            (2, PosY, NegZ) => 0,
            (2, PosY, PosX) => 0,
            (2, PosY, PosY) => 2,
            (2, PosY, PosZ) => 0,
            (2, PosZ, NegX) => 1,
            (2, PosZ, NegY) => 0,
            (2, PosZ, NegZ) => 0,
            (2, PosZ, PosX) => 3,
            (2, PosZ, PosY) => 2,
            (2, PosZ, PosZ) => 2,
            (3, NegX, NegX) => 3,
            (3, NegX, NegY) => 3,
            (3, NegX, NegZ) => 1,
            (3, NegX, PosX) => 3,
            (3, NegX, PosY) => 3,
            (3, NegX, PosZ) => 3,
            (3, NegY, NegX) => 2,
            (3, NegY, NegY) => 3,
            (3, NegY, NegZ) => 2,
            (3, NegY, PosX) => 2,
            (3, NegY, PosY) => 1,
            (3, NegY, PosZ) => 2,
            (3, NegZ, NegX) => 3,
            (3, NegZ, NegY) => 2,
            (3, NegZ, NegZ) => 3,
            (3, NegZ, PosX) => 1,
            (3, NegZ, PosY) => 0,
            (3, NegZ, PosZ) => 3,
            (3, PosX, NegX) => 3,
            (3, PosX, NegY) => 1,
            (3, PosX, NegZ) => 3,
            (3, PosX, PosX) => 3,
            (3, PosX, PosY) => 1,
            (3, PosX, PosZ) => 1,
            (3, PosY, NegX) => 0,
            (3, PosY, NegY) => 1,
            (3, PosY, NegZ) => 0,
            (3, PosY, PosX) => 0,
            (3, PosY, PosY) => 3,
            (3, PosY, PosZ) => 0,
            (3, PosZ, NegX) => 1,
            (3, PosZ, NegY) => 0,
            (3, PosZ, NegZ) => 3,
            (3, PosZ, PosX) => 3,
            (3, PosZ, PosY) => 2,
            (3, PosZ, PosZ) => 3,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rotation(up={},forward={},angle={})", self.up(), self.forward(), self.angle())
    }
}