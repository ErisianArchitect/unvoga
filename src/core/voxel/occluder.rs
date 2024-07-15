use crate::core::math::coordmap::Rotation;

use super::{direction::Direction, occlusion_shape::OcclusionShape};

pub struct Occluder {
    pub neg_x: OcclusionShape,
    pub neg_y: OcclusionShape,
    pub neg_z: OcclusionShape,
    pub pos_x: OcclusionShape,
    pub pos_y: OcclusionShape,
    pub pos_z: OcclusionShape,
}

impl Occluder {
    pub const EMPTY_FACES: Occluder = Occluder::new(
        OcclusionShape::Empty,
        OcclusionShape::Empty,
        OcclusionShape::Empty,
        OcclusionShape::Empty,
        OcclusionShape::Empty,
        OcclusionShape::Empty
    );
    pub const FULL_FACES: Occluder = Occluder::new(
        OcclusionShape::Full,
        OcclusionShape::Full,
        OcclusionShape::Full,
        OcclusionShape::Full,
        OcclusionShape::Full,
        OcclusionShape::Full
    );

    #[inline(always)]
    pub const fn new(
        neg_x: OcclusionShape, neg_y: OcclusionShape, neg_z: OcclusionShape,
        pos_x: OcclusionShape, pos_y: OcclusionShape, pos_z: OcclusionShape
    ) -> Self {
        Self {
            neg_x, neg_y, neg_z,
            pos_x, pos_y, pos_z
        }
    }
    
    #[inline(always)]
    pub fn face(&self, face: Direction) -> &OcclusionShape {
        match face {
            Direction::NegX => &self.neg_x,
            Direction::NegY => &self.neg_y,
            Direction::NegZ => &self.neg_z,
            Direction::PosX => &self.pos_x,
            Direction::PosY => &self.pos_y,
            Direction::PosZ => &self.pos_z,
        }
    }

    #[inline(always)]
    pub fn face_mut(&mut self, face: Direction) -> &mut OcclusionShape {
        match face {
            Direction::NegX => &mut self.neg_x,
            Direction::NegY => &mut self.neg_y,
            Direction::NegZ => &mut self.neg_z,
            Direction::PosX => &mut self.pos_x,
            Direction::PosY => &mut self.pos_y,
            Direction::PosZ => &mut self.pos_z,
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (Direction, &OcclusionShape)> {
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

    #[inline(always)]
    pub fn occluded_by(&self, rotation: Rotation, face: Direction, other: &Occluder, other_rotation: Rotation) -> bool {
        let other_face = face.invert();
        let face_angle = rotation.face_angle(face);
        let other_face_angle = rotation.face_angle(other_face);
        let source_face = rotation.source_face(face);
        let other_source_face = rotation.source_face(other_face);
        let occluder = self.face(source_face);
        let other_occluder = other.face(other_source_face);
        occluder.occluded_by(other_occluder, face_angle, other_face_angle)
    }
}

impl std::ops::Index<Direction> for Occluder {
    type Output = OcclusionShape;
    fn index(&self, index: Direction) -> &Self::Output {
        self.face(index)
    }
}

impl std::ops::IndexMut<Direction> for Occluder {
    fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
        self.face_mut(index)
    }
}