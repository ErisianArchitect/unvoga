#![allow(unused)]
use crate::prelude::*;

use super::{direction::Direction, occlusionshape::OcclusionShape};

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

    
    pub const fn new(
        neg_x: OcclusionShape, neg_y: OcclusionShape, neg_z: OcclusionShape,
        pos_x: OcclusionShape, pos_y: OcclusionShape, pos_z: OcclusionShape
    ) -> Self {
        Self {
            neg_x, neg_y, neg_z,
            pos_x, pos_y, pos_z
        }
    }
    
    
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

    pub fn occluded_by(&self, orientation: Orientation, face: Direction, other: &Self, other_orientation: Orientation) -> bool {
        let other_face = face.invert();
        let occl_face = orientation.source_face(face);
        let l_occl = self.face(occl_face);
        let occl_face = other_orientation.source_face(other_face);
        let r_occl = other.face(occl_face);
        if r_occl.is_empty() {
            return false;
        }
        if r_occl.is_full() {
            return !l_occl.is_empty();
        }
        match l_occl {
            OcclusionShape::Full => match r_occl {
                OcclusionShape::S16x16(shape) => shape.0.iter().find(|&&sub| sub != u16::MAX).is_none(),
                OcclusionShape::S8x8(shape) => shape.0 == u64::MAX,
                OcclusionShape::S4x4(shape) => shape.0 == u16::MAX,
                OcclusionShape::S2x2(shape) => shape.0 & 0xF == 0xF,
                OcclusionShape::Rect(shape) => *shape == OcclusionRect::FULL,
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::Rect(shape) => match r_occl {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    let shape = shape.transform_face(orientation, face);
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            let (ox, oy) = (x as i8 - 8, y as i8 - 8);
                            let (ox, oy) = other_orientation.source_face_coord(other_face, (ox, oy));
                            let (ox, oy) = ((ox + 8) as usize, (oy + 8) as usize);
                            if !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S8x8(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    let shape = shape.transform_face(orientation, face);
                    let sample = shape.downsample(8);
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            let (ox, oy) = (x as i8 - 4, y as i8 - 4);
                            let (ox, oy) = other_orientation.source_face_coord(other_face, (ox, oy));
                            let (ox, oy) = ((ox + 4) as usize, (oy + 4) as usize);
                            if !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    let shape = shape.transform_face(orientation, face);
                    let sample = shape.downsample(4);
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            let (ox, oy) = (x as i8 - 2, y as i8 - 2);
                            let (ox, oy) = other_orientation.source_face_coord(other_face, (ox, oy));
                            let (ox, oy) = ((ox + 2) as usize, (oy + 2) as usize);
                            if !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    let shape = shape.transform_face(orientation, face);
                    let sample = shape.downsample(2);
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            let (ox, oy) = (x as i8 - 1, y as i8 - 1);
                            let (ox, oy) = other_orientation.source_face_coord(other_face, (ox, oy));
                            let (ox, oy) = ((ox + 1) as usize, (oy + 1) as usize);
                            if !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    let shape = shape.transform_face(orientation, face);
                    let other = other.transform_face(other_orientation, other_face);
                    other.contains_rect(shape)
                },
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::S16x16(shape) => match r_occl {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    for y in 0..16 {
                        for x in 0..16 {
                            let (sx, sy) = (x as i8 - 8, y as i8 - 8);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 8) as usize, (sy + 8) as usize);
                            let (ox, oy) = (x as i8 - 8, y as i8 - 8);
                            let (ox, oy) = other_orientation.source_face_coord(other_face, (ox, oy));
                            let (ox, oy) = ((ox + 8) as usize, (oy + 8) as usize);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S8x8(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    for y in 0..16 {
                        let oy = y / 2;
                        for x in 0..16 {
                            let ox = x / 2;
                            let (sx, sy) = (x as i8 - 8, y as i8 - 8);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 8) as usize, (sy + 8) as usize);
                            let (nox, noy) = (ox as i8 - 4, oy as i8 - 4);
                            let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                            let (nox, noy) = ((nox + 4) as usize, (noy + 4) as usize);
                            if shape.get(sx, sy) && !other.get(nox, noy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    for y in 0..16 {
                        let oy = y / 4;
                        for x in 0..16 {
                            let ox = x / 4;
                            let (sx, sy) = (x as i8 - 8, y as i8 - 8);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 8) as usize, (sy + 8) as usize);
                            let (nox, noy) = (ox as i8 - 2, oy as i8 - 2);
                            let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                            let (nox, noy) = ((nox + 2) as usize, (noy + 2) as usize);
                            if shape.get(sx, sy) && !other.get(nox, noy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    for y in 0..16 {
                        let oy = y / 8;
                        for x in 0..16 {
                            let ox = x / 8;
                            let (sx, sy) = (x as i8 - 8, y as i8 - 8);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 8) as usize, (sy + 8) as usize);
                            let (nox, noy) = (ox as i8 - 1, oy as i8 - 1);
                            let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                            let (nox, noy) = ((nox + 1) as usize, (noy + 1) as usize);
                            if shape.get(sx, sy) && !other.get(nox, noy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    let other = other.transform_face(other_orientation, other_face);
                    for y in 0..16 {
                        for x in 0..16 {
                            let (sx, sy) = (x as i8 - 8, y as i8 - 8);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 8) as usize, (sy + 8) as usize);
                            if shape.get(sx, sy) && !other.contains((x as u8, y as u8)) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::S8x8(shape) => match r_occl {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    for y in 0..8 {
                        let oy = y * 2;
                        for x in 0..8 {
                            let (sx, sy) = (x as i8 - 4, y as i8 - 4);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 4) as usize, (sy + 4) as usize);
                            if shape.get(sx, sy) {
                                let ox = x * 2;
                                for oy in oy..oy+2 {
                                    for ox in ox..ox+2 {
                                        let (nox, noy) = (ox as i8 - 8, oy as i8 - 8);
                                        let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                                        let (nox, noy) = ((nox + 8) as usize, (noy + 8) as usize);
                                        if !other.get(nox, noy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },            
                OcclusionShape::S8x8(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    for y in 0..8 {
                        for x in 0..8 {
                            let (sx, sy) = (x as i8 - 4, y as i8 - 4);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 4) as usize, (sy + 4) as usize);
                            let (ox, oy) = (x as i8 - 4, y as i8 - 4);
                            let (ox, oy) = other_orientation.source_face_coord(other_face, (ox, oy));
                            let (ox, oy) = ((ox + 4) as usize, (oy + 4) as usize);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    for y in 0..8 {
                        let oy = y / 2;
                        for x in 0..8 {
                            let ox = x / 2;
                            let (sx, sy) = (x as i8 - 4, y as i8 - 4);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 4) as usize, (sy + 4) as usize);
                            let (nox, noy) = (ox as i8 - 2, oy as i8 - 2);
                            let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                            let (nox, noy) = ((nox + 2) as usize, (noy + 2) as usize);
                            if shape.get(sx, sy) && !other.get(nox, noy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    for y in 0..8 {
                        let oy = y / 4;
                        for x in 0..8 {
                            let ox = x / 4;
                            let (sx, sy) = (x as i8 - 4, y as i8 - 4);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 4) as usize, (sy + 4) as usize);
                            let (nox, noy) = (ox as i8 - 1, oy as i8 - 1);
                            let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                            let (nox, noy) = ((nox + 1) as usize, (noy + 1) as usize);
                            if shape.get(sx, sy) && !other.get(nox, noy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    let other = other.transform_face(other_orientation, other_face);
                    for y in 0..8 {
                        for x in 0..8 {
                            let (sx, sy) = (x as i8 - 4, y as i8 - 4);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 4) as usize, (sy + 4) as usize);
                            if shape.get(sx, sy) {
                                let inner = OcclusionRect::from_min_max(
                                    (x as u8 * 2, y as u8 * 2),
                                    (x as u8 * 2 + 2, y as u8 * 2 + 2)
                                );
                                if !other.contains_rect(inner) {
                                    return false;
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::S4x4(shape) => match r_occl {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    for y in 0..4 {
                        let oy = y * 4;
                        for x in 0..4 {
                            let (sx, sy) = (x as i8 - 2, y as i8 - 2);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 2) as usize, (sy + 2) as usize);
                            if shape.get(sx, sy) {
                                let ox = x * 4;
                                for oy in oy..oy+4 {
                                    for ox in ox..ox+4 {
                                        let (nox, noy) = (ox as i8 - 8, oy as i8 - 8);
                                        let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                                        let (nox, noy) = ((nox + 8) as usize, (noy + 8) as usize);
                                        if !other.get(nox, noy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S8x8(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    for y in 0..4 {
                        let oy = y * 2;
                        for x in 0..4 {
                            let (sx, sy) = (x as i8 - 2, y as i8 - 2);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 2) as usize, (sy + 2) as usize);
                            if shape.get(sx, sy) {
                                let ox = x * 2;
                                for oy in oy..oy+2 {
                                    for ox in ox..ox+2 {
                                        let (nox, noy) = (ox as i8 - 4, oy as i8 - 4);
                                        let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                                        let (nox, noy) = ((nox + 4) as usize, (noy + 4) as usize);
                                        if !other.get(nox, noy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    for y in 0..4 {
                        for x in 0..4 {
                            let (sx, sy) = (x as i8 - 2, y as i8 - 2);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 2) as usize, (sy + 2) as usize);
                            let (ox, oy) = (x as i8 - 2, y as i8 - 2);
                            let (ox, oy) = other_orientation.source_face_coord(other_face, (ox, oy));
                            let (ox, oy) = ((ox + 2) as usize, (oy + 2) as usize);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    for y in 0..4 {
                        let oy = y / 2;
                        for x in 0..4 {
                            let ox = x / 2;
                            let (sx, sy) = (x as i8 - 2, y as i8 - 2);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 2) as usize, (sy + 2) as usize);
                            let (nox, noy) = (ox as i8 - 1, oy as i8 - 1);
                            let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                            let (nox, noy) = ((nox + 1) as usize, (noy + 1) as usize);
                            if shape.get(sx, sy) && !other.get(nox, noy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    let other = other.transform_face(other_orientation, other_face);
                    for y in 0..4 {
                        for x in 0..4 {
                            let (sx, sy) = (x as i8 - 2, y as i8 - 2);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 2) as usize, (sy + 2) as usize);
                            if shape.get(sx, sy) {
                                let inner = OcclusionRect::from_min_max(
                                    (x as u8 * 4, y as u8 * 4),
                                    (x as u8 * 4 + 4, y as u8 * 4 + 4)
                                );
                                if !other.contains_rect(inner) {
                                    return false;
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::S2x2(shape) => match r_occl {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    for y in 0..2 {
                        let oy = y * 8;
                        for x in 0..2 {
                            let (sx, sy) = (x as i8 - 1, y as i8 - 1);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 1) as usize, (sy + 1) as usize);
                            if shape.get(sx, sy) {
                                let ox = x * 8;
                                for oy in oy..oy+8 {
                                    for ox in ox..ox+8 {
                                        let (nox, noy) = (ox as i8 - 8, oy as i8 - 8);
                                        let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                                        let (nox, noy) = ((nox + 8) as usize, (noy + 8) as usize);
                                        if !other.get(nox, noy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S8x8(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    for y in 0..2 {
                        let oy = y * 4;
                        for x in 0..2 {
                            let (sx, sy) = (x as i8 - 1, y as i8 - 1);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 1) as usize, (sy + 1) as usize);
                            if shape.get(sx, sy) {
                                let ox = x * 4;
                                for oy in oy..oy+4 {
                                    for ox in ox..ox+4 {
                                        let (nox, noy) = (ox as i8 - 4, oy as i8 - 4);
                                        let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                                        let (nox, noy) = ((nox + 4) as usize, (noy + 4) as usize);
                                        if !other.get(nox, noy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    for y in 0..2 {
                        let oy = y * 2;
                        for x in 0..2 {
                            let (sx, sy) = (x as i8 - 1, y as i8 - 1);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 1) as usize, (sy + 1) as usize);
                            if shape.get(sx, sy) {
                                let ox = x * 2;
                                for oy in oy..oy+2 {
                                    for ox in ox..ox+2 {
                                        let (nox, noy) = (ox as i8 - 2, oy as i8 - 2);
                                        let (nox, noy) = other_orientation.source_face_coord(other_face, (nox, noy));
                                        let (nox, noy) = ((nox + 2) as usize, (noy + 2) as usize);
                                        if !other.get(nox, noy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    for y in 0..2 {
                        for x in 0..2 {
                            let (sx, sy) = (x as i8 - 1, y as i8 - 1);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 1) as usize, (sy + 1) as usize);
                            let (ox, oy) = (x as i8 - 1, y as i8 - 1);
                            let (ox, oy) = other_orientation.source_face_coord(other_face, (ox, oy));
                            let (ox, oy) = ((ox + 1) as usize, (oy + 1) as usize);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    let other = other.transform_face(other_orientation, other_face);
                    for y in 0..2 {
                        for x in 0..2 {
                            let (sx, sy) = (x as i8 - 1, y as i8 - 1);
                            let (sx, sy) = orientation.source_face_coord(face, (sx, sy));
                            let (sx, sy) = ((sx + 1) as usize, (sy + 1) as usize);
                            if shape.get(sx, sy) {
                                let inner = OcclusionRect::from_min_max(
                                    (x as u8 * 8, y as u8 * 8),
                                    (x as u8 * 8 + 8, y as u8 * 8 + 8)
                                );
                                if !other.contains_rect(inner) {
                                    return false;
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::Empty => {
                r_occl.fully_occluded()
            },
        }
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