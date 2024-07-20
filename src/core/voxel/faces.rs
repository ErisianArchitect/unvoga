use std::ops::{Index, IndexMut};

use super::direction::Direction;
use paste::paste;

pub struct Faces<T> {
    pub neg_x: T,
    pub neg_y: T,
    pub neg_z: T,
    pub pos_x: T,
    pub pos_y: T,
    pub pos_z: T,
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
            neg_x,
            neg_y,
            neg_z,
            pos_x,
            pos_y,
            pos_z,
        }
    }

    
    pub const fn face(&self, direction: Direction) -> &T {
        match direction {
            Direction::NegX => &self.neg_x,
            Direction::NegY => &self.neg_y,
            Direction::NegZ => &self.neg_z,
            Direction::PosX => &self.pos_x,
            Direction::PosY => &self.pos_y,
            Direction::PosZ => &self.pos_z,
        }
    }


    
    pub fn face_mut(&mut self, direction: Direction) -> &mut T {
        match direction {
            Direction::NegX => &mut self.neg_x,
            Direction::NegY => &mut self.neg_y,
            Direction::NegZ => &mut self.neg_z,
            Direction::PosX => &mut self.pos_x,
            Direction::PosY => &mut self.pos_y,
            Direction::PosZ => &mut self.pos_z,
        }
    }

    
    pub fn iter(&self) -> impl Iterator<Item = (Direction, &T)> {
        use Direction::*;
        [
            (NegX, &self.neg_x),
            (NegY, &self.neg_y),
            (NegZ, &self.neg_z),
            (PosX, &self.pos_x),
            (PosY, &self.pos_y),
            (PosZ, &self.pos_z),
        ].into_iter()
    }
}

impl<T> std::ops::Index<Direction> for Faces<T> {
    type Output = T;

    
    fn index(&self, index: Direction) -> &Self::Output {
        self.face(index)
    }
}

impl<T> std::ops::IndexMut<Direction> for Faces<T> {
    
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        self.face_mut(index)
    }
}

impl<T: Clone> Clone for Faces<T> {
    fn clone(&self) -> Self {
        Self {
            neg_x: self.neg_x.clone(),
            neg_y: self.neg_y.clone(),
            neg_z: self.neg_z.clone(),
            pos_x: self.pos_x.clone(),
            pos_y: self.pos_y.clone(),
            pos_z: self.pos_z.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.neg_x = source.neg_x.clone();
        self.neg_y = source.neg_y.clone();
        self.neg_z = source.neg_z.clone();
        self.pos_x = source.pos_x.clone();
        self.pos_y = source.pos_y.clone();
        self.pos_z = source.pos_z.clone();
    }
}

impl<T: Copy> Copy for Faces<T> {}