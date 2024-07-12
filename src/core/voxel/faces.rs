use std::ops::{Index, IndexMut};

use super::direction::Direction;
use paste::paste;

/// A simple storage container for data associated with the 6 sides of a cube.
pub struct Faces<T> {
    pub faces: [T; 6],
    // /// (-1, 0, 0)
    // pub neg_x: T,
    // /// (0, -1, 0)
    // pub neg_y: T,
    // /// (0, 0, -1)
    // pub neg_z: T,
    // /// (1, 0, 0)
    // pub pos_x: T,
    // /// (0, 1, 0)
    // pub pos_y: T,
    // /// (0, 0, 1)
    // pub pos_z: T
}

macro_rules! faces_getters {
    ($($name:ident[$direction:expr];)*) => {
        $(
            paste! {
                #[inline]
                pub fn $name(&self) -> &T {
                    &self.faces[$direction as usize]
                }

                #[inline]
                pub fn [<$name _mut>](&mut self) -> &mut T {
                    &mut self.faces[$direction as usize]
                }
            }
        )*
    };
}

impl<T> Faces<T> {
    pub const fn new(
        neg_x: T,
        neg_y: T,
        neg_z: T,
        pos_x: T,
        pos_y: T,
        pos_z: T
    ) -> Self {
        Self {
            faces: [
                pos_y,
                pos_x,
                pos_z,
                neg_y,
                neg_x,
                neg_z
            ]
        }
    }

    faces_getters!(
        neg_x[Direction::NegX];
        neg_y[Direction::NegY];
        neg_z[Direction::NegZ];
        pos_x[Direction::PosX];
        pos_y[Direction::PosY];
        pos_z[Direction::PosZ];
    );

    pub fn iter(&self) -> impl Iterator<Item = (Direction, &T)> {
        Direction::iter().map(|dir| {
            (dir, &self.faces[dir as usize])
        })
    }
}

impl<T> Index<Direction> for Faces<T> {
    type Output = T;
    fn index(&self, index: Direction) -> &Self::Output {
        &self.faces[index as usize]
    }
}

impl<T> IndexMut<Direction> for Faces<T> {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        &mut self.faces[index as usize]
    }
}

#[test]
pub fn faces_test() {
    let faces = Faces::new(0, 0, 0, 0, 0, 0);
    faces[Direction::NegX];
}

impl<T: Clone> Clone for Faces<T> {
    fn clone(&self) -> Self {
        Self {
            faces: self.faces.clone()
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.faces = source.faces.clone();
    }
}

impl<T: Copy> Copy for Faces<T> {}