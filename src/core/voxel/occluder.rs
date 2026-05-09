#![allow(unused)]
use crate::prelude::*;

use super::{direction::Direction, occlusionshape::OcclusionShape};

/// Expand an 8-bit row to a 16-bit row by doubling each bit:
/// bit i in the input becomes bits 2i and 2i+1 in the output.
/// Used for S8x8 -> S16x16 mask expansion in fast paths.
#[inline]
fn expand_byte_to_u16(b: u8) -> u16 {
    let mut x = b as u16;
    x = (x | (x << 4)) & 0x0F0F;
    x = (x | (x << 2)) & 0x3333;
    x = (x | (x << 1)) & 0x5555;
    x | (x << 1)
}

/// Row-mask for an OcclusionRect at row `y` in 16x16 coord space.
/// Returns 0 if y is outside [top, bottom); otherwise a u16 with bits
/// set in [left, right).
#[inline]
fn rect_row_mask(rect: OcclusionRect, y: u8) -> u16 {
    if y < rect.top || y >= rect.bottom {
        return 0;
    }
    let lo = rect.left as u32;
    let hi = rect.right as u32;
    ((1u32 << hi).wrapping_sub(1u32 << lo)) as u16
}

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
                    // Fast path: default orientations -> row-mask AND-NOT.
                    if orientation == Orientation::default() && other_orientation == Orientation::default() {
                        return (0..16u8).all(|y| {
                            (rect_row_mask(*shape, y) & !other.0[y as usize]) == 0
                        });
                    }
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
                    for y in sample.top..sample.bottom {
                        for x in sample.left..sample.right {
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
                    for y in sample.top..sample.bottom {
                        for x in sample.left..sample.right {
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
                    for y in sample.top..sample.bottom {
                        for x in sample.left..sample.right {
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
                    // Fast path: default orientations skip per-pixel transform
                    // and reduce to row-wise bitwise compare.
                    // ~80x faster than per-pixel loop.
                    if orientation == Orientation::default() && other_orientation == Orientation::default() {
                        return shape.0.iter().zip(other.0.iter())
                            .all(|(s, o)| (s & !o) == 0);
                    }
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
                    // Fast path: default orientations -> expand other(S8) to 16x16
                    // row-by-row and bitwise AND-NOT.
                    if orientation == Orientation::default() && other_orientation == Orientation::default() {
                        return (0..16usize).all(|y| {
                            let byte = ((other.0 >> ((y / 2) * 8)) & 0xFF) as u8;
                            (shape.0[y] & !expand_byte_to_u16(byte)) == 0
                        });
                    }
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
                    // Fast path: default orientations -> per-row mask AND-NOT.
                    if orientation == Orientation::default() && other_orientation == Orientation::default() {
                        return (0..16u8).all(|y| {
                            (shape.0[y as usize] & !rect_row_mask(*other, y)) == 0
                        });
                    }
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
                    // Fast path: default orientations -> expand S8 row to 16 wide,
                    // bitwise AND-NOT each S16 row.
                    if orientation == Orientation::default() && other_orientation == Orientation::default() {
                        return (0..16usize).all(|y| {
                            let byte = ((shape.0 >> ((y / 2) * 8)) & 0xFF) as u8;
                            (expand_byte_to_u16(byte) & !other.0[y]) == 0
                        });
                    }
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
                    // Fast path: default orientations -> single u64 AND-NOT.
                    if orientation == Orientation::default() && other_orientation == Orientation::default() {
                        return (shape.0 & !other.0) == 0;
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::voxel::occlusionshape::{
        OcclusionShape, OcclusionShape16x16, OcclusionShape8x8,
    };

    fn occluder_with(s: OcclusionShape) -> Occluder {
        Occluder::new(s.clone(), s.clone(), s.clone(), s.clone(), s.clone(), s)
    }

    #[test]
    fn expand_byte_matches_naive() {
        for b in 0u16..256 {
            let b = b as u8;
            let expected: u16 = (0..8).fold(0u16, |acc, i| {
                if b & (1 << i) != 0 { acc | (0b11u16 << (i * 2)) } else { acc }
            });
            assert_eq!(expand_byte_to_u16(b), expected, "byte {b:08b}");
        }
    }

    #[test]
    fn rect_row_mask_basic() {
        let r = OcclusionRect::new(2, 3, 5, 4); // left=2 right=7 top=3 bottom=7
        assert_eq!(rect_row_mask(r, 2), 0);
        assert_eq!(rect_row_mask(r, 3), 0b0000_0000_0111_1100);
        assert_eq!(rect_row_mask(r, 6), 0b0000_0000_0111_1100);
        assert_eq!(rect_row_mask(r, 7), 0);
        let full = OcclusionRect::new(0, 0, 16, 16);
        for y in 0..16 {
            assert_eq!(rect_row_mask(full, y), 0xFFFF, "y={y}");
        }
    }

    /// Slow-path reference: per-pixel iteration without the bitwise fast-path.
    fn s16_vs_s16_slow(
        shape: &OcclusionShape16x16,
        other: &OcclusionShape16x16,
        orientation: Orientation,
        face: Direction,
        other_orientation: Orientation,
        other_face: Direction,
    ) -> bool {
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
    }

    /// Fast-path equivalence: default orientation should match slow-path.
    #[test]
    fn s16_fast_path_matches_slow_path() {
        let cases: Vec<([u16; 16], [u16; 16])> = vec![
            ([u16::MAX; 16], [u16::MAX; 16]),
            ([u16::MAX; 16], [0; 16]),
            ([0; 16], [u16::MAX; 16]),
            ([0xAAAA; 16], [0xAAAA; 16]),
            ([0xAAAA; 16], [0x5555; 16]),
            ([0x00FF; 16], [0x0F0F; 16]),
        ];
        let o = Orientation::default();
        for face in [Direction::PosY, Direction::NegX, Direction::PosZ] {
            let other_face = face.invert();
            for (s, ot) in &cases {
                let s = OcclusionShape16x16::new(*s);
                let ot = OcclusionShape16x16::new(*ot);
                let a = occluder_with(OcclusionShape::S16x16(s));
                let b = occluder_with(OcclusionShape::S16x16(ot));
                let fast = a.occluded_by(o, face, &b, o);
                let slow = s16_vs_s16_slow(&s, &ot, o, face, o, other_face);
                assert_eq!(fast, slow, "face={face:?} s={:04x?} o={:04x?}", s.0, ot.0);
            }
        }
    }
}