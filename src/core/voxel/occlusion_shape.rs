/*
The OcclusionShape should be used for light and mesh occlusion.
You should logically map the x and y coordinates to the x/y/z coordinates.
If you're making an occluder on the X axis, use Z for X and Y for Y.
If you're making an occluder on the Y axis, use X for X and Z for Y.
If you're making an occluder on the Z axis, use X for X and Y for Y.

If you want to rotate an occluder, good luck.
*/

use super::faces::Faces;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OcclusionShape16x16([u16; 16]);

impl OcclusionShape16x16 {
    pub fn get(&self, x: usize, y: usize) -> bool {
        let sub = self.0[y];
        sub & (1 << x) != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        let sub = self.0[y];
        self.0[y] = if value {
            sub | (1 << x)
        } else {
            sub & !(1 << x)
        };
        sub & (1 << x) != 0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OcclusionShape8x8(u64);

impl OcclusionShape8x8 {
    pub fn get(self, x: usize, y: usize) -> bool {
        let index = y * 8 + x;
        self.0 & (1 << index) != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        let index = y * 8 + x;
        let old = self.0 & (1 << index) != 0;
        self.0 = if value {
            self.0 | (1 << index)
        } else {
            self.0 & !(1 << index)
        };
        old
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OcclusionShape4x4(u16);

impl OcclusionShape4x4 {
    pub fn get(self, x: usize, y: usize) -> bool {
        let index = y * 4 + x;
        self.0 & (1 << index) != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        let index = y * 4 + x;
        let old = self.0 & (1 << index) != 0;
        self.0 = if value {
            self.0 | (1 << index)
        } else {
            self.0 & !(1 << index)
        };
        old
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OcclusionShape2x2(u8);

impl OcclusionShape2x2 {
    pub fn get(self, x: usize, y: usize) -> bool {
        let index = y * 2 + x;
        self.0 & (1 << index) != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        let index = y * 2 + x;
        let old = self.0 & (1 << index) != 0;
        self.0 = if value {
            self.0 | (1 << index)
        } else {
            self.0 & !(1 << index)
        };
        old
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OcclusionRect {
    left: u8,
    top: u8,
    right: u8,
    bottom: u8
}

impl OcclusionRect {
    const FULL: OcclusionRect = OcclusionRect {
        left: 0,
        top: 0,
        right: 16,
        bottom: 16
    };
    pub fn new(x: u8, y: u8, width: u8, height: u8) -> Self {
        Self {
            left: x,
            top: y,
            right: x.checked_add(width).expect("Overflow on X axis"),
            bottom: y.checked_add(height).expect("Overflow on Y axis")
        }
    }

    pub fn from_min_max(min: (u8, u8), max: (u8, u8)) -> Self {
        Self {
            left: min.0,
            top: min.1,
            right: max.0,
            bottom: max.1
        }
    }

    pub fn contains(self, point: (u8, u8)) -> bool {
        self.left <= point.0
        && self.right > point.0
        && self.top <= point.1
        && self.bottom > point.1
    }

    pub fn intersects(self, other: OcclusionRect) -> bool {
        self.left < other.right
        && other.left < self.right
        && self.top < other.bottom
        && other.top < self.bottom
    }

    pub fn contains_rect(self, other: OcclusionRect) -> bool {
        self.left <= other.left
        && self.top <= other.top
        && self.right >= other.right
        && self.bottom >= other.bottom
    }

    /// Reduces the size of the rectangle to fit within the sample size. This stretches the bounds to fit the most space.
    pub fn downsample(self, sample_size: u8) -> Self {
        fn reduce_min(value: u8, reduction: u8) -> u8 {
            value / (16 / reduction)
        }
        fn reduce_max(value: u8, reduction: u8) -> u8 {
            let div = 16 / reduction;
            value / div + if value % div == 0 { 0 } else { 1 }
        }
        Self {
            left: reduce_min(self.left, sample_size),
            top: reduce_min(self.top, sample_size),
            right: reduce_max(self.right, sample_size),
            bottom: reduce_max(self.bottom, sample_size)
        }
    }
}

pub enum OcclusionShape {
    S16x16(OcclusionShape16x16),
    S8x8(OcclusionShape8x8),
    S4x4(OcclusionShape4x4),
    S2x2(OcclusionShape2x2),
    Rect(OcclusionRect),
    Full,
    None
}

impl OcclusionShape {
    pub const FULL_FACES: Faces<OcclusionShape> = Faces {
        neg_x: OcclusionShape::Full,
        neg_y: OcclusionShape::Full,
        neg_z: OcclusionShape::Full,
        pos_x: OcclusionShape::Full,
        pos_y: OcclusionShape::Full,
        pos_z: OcclusionShape::Full,
    };
    pub const EMPTY_FACES: Faces<OcclusionShape> = Faces {
        neg_x: OcclusionShape::None,
        neg_y: OcclusionShape::None,
        neg_z: OcclusionShape::None,
        pos_x: OcclusionShape::None,
        pos_y: OcclusionShape::None,
        pos_z: OcclusionShape::None,
    };
    pub fn is_none(&self) -> bool {
        matches!(self, OcclusionShape::None)
    }

    pub fn is_full(&self) -> bool {
        matches!(self, OcclusionShape::Full)
    }

    pub fn occludes(&self) -> bool {
        match self {
            OcclusionShape::S16x16(shape) => shape.0.iter().find(|&&sub| sub != 0).is_some(),
            OcclusionShape::S8x8(shape) => shape.0 != 0,
            OcclusionShape::S4x4(shape) => shape.0 != 0,
            OcclusionShape::S2x2(shape) => shape.0 != 0,
            OcclusionShape::Rect(shape) => true,
            OcclusionShape::Full => true,
            OcclusionShape::None => false,
        }
    }

    pub fn occluded_by(&self, other: &OcclusionShape) -> bool {
        // What I wanted to do for occlusion is rather complicated combinatorically, so nested match
        // expressions is the way to go. There may be a better way to do it, but I'm not smart enough to know it.
        if other.is_none()
        || self.is_none() {
            return false;
        }
        if other.is_full() {
            return self.occludes();
        }
        match self {
            OcclusionShape::Full => match other {
                OcclusionShape::S16x16(shape) => shape.0.iter().find(|&&sub| sub != u16::MAX).is_none(),
                OcclusionShape::S8x8(shape) => shape.0 == u64::MAX,
                OcclusionShape::S4x4(shape) => shape.0 == u16::MAX,
                OcclusionShape::S2x2(shape) => shape.0 & 0xF == 0xF,
                OcclusionShape::Rect(shape) => *shape == OcclusionRect::FULL,
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::None => unreachable!(),
            },
            OcclusionShape::Rect(shape) => match other {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            if !other.get(x as usize, y as usize) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S8x8(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    let sample = shape.downsample(8);
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            if !other.get(x as usize, y as usize) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    let sample = shape.downsample(4);
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            if !other.get(x as usize, y as usize) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    let sample = shape.downsample(2);
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            if !other.get(x as usize, y as usize) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => other.contains_rect(*shape),
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::None => unreachable!(),
            },
            OcclusionShape::S16x16(shape) => match other {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    for y in 0..16 {
                        if shape.0[y] & other.0[y] != shape.0[y] {
                            return false;
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
                            if shape.get(x, y) && !other.get(ox, oy) {
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
                            if shape.get(x, y) && !other.get(ox, oy) {
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
                            if shape.get(x, y) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    for y in 0..16 {
                        for x in 0..16 {
                            if shape.get(x, y) && !other.contains((x as u8, y as u8)) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::None => unreachable!(),
            },
            OcclusionShape::S8x8(shape) => match other {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    for y in 0..8 {
                        let oy = y * 2;
                        for x in 0..8 {
                            if shape.get(x, y) {
                                let ox = x * 2;
                                for oy in oy..oy+2 {
                                    for ox in ox..ox+2 {
                                        if !other.get(ox, oy) {
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
                    shape.0 != 0 && shape.0 & other.0 == shape.0
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    for y in 0..8 {
                        let oy = y / 2;
                        for x in 0..8 {
                            let ox = x / 2;
                            if shape.get(x, y) && !other.get(ox, oy) {
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
                            if shape.get(x, y) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    for y in 0..8 {
                        for x in 0..8 {
                            if shape.get(x, y) {
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
                OcclusionShape::None => unreachable!(),
            },
            OcclusionShape::S4x4(shape) => match other {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    for y in 0..4 {
                        let oy = y * 4;
                        for x in 0..4 {
                            if shape.get(x, y) {
                                let ox = x * 4;
                                for oy in oy..oy+4 {
                                    for ox in ox..ox+4 {
                                        if !other.get(ox, oy) {
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
                            if shape.get(x, y) {
                                let ox = x * 2;
                                for oy in oy..oy+2 {
                                    for ox in ox..ox+2 {
                                        if !other.get(ox, oy) {
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
                    shape.0 != 0 && shape.0 & other.0 == shape.0
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    for y in 0..4 {
                        let oy = y / 2;
                        for x in 0..4 {
                            let ox = x / 2;
                            if shape.get(x, y) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    for y in 0..4 {
                        for x in 0..4 {
                            if shape.get(x, y) {
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
                OcclusionShape::None => unreachable!(),
            },
            OcclusionShape::S2x2(shape) => match other {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    for y in 0..2 {
                        let oy = y * 8;
                        for x in 0..2 {
                            if shape.get(x, y) {
                                let ox = x * 8;
                                for oy in oy..oy+8 {
                                    for ox in ox..ox+8 {
                                        if !other.get(ox, oy) {
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
                            if shape.get(x, y) {
                                let ox = x * 4;
                                for oy in oy..oy+4 {
                                    for ox in ox..ox+4 {
                                        if !other.get(ox, oy) {
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
                            if shape.get(x, y) {
                                let ox = x * 2;
                                for oy in oy..oy+2 {
                                    for ox in ox..ox+2 {
                                        if !other.get(ox, oy) {
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
                    shape.0 != 0 && shape.0 & other.0 == shape.0
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    for y in 0..2 {
                        for x in 0..2 {
                            if shape.get(x, y) {
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
                OcclusionShape::None => unreachable!(),
            },
            OcclusionShape::None => unreachable!(),
        }
    }
}

impl From<[[u8; 2]; 2]> for OcclusionShape {
    fn from(value: [[u8; 2]; 2]) -> Self {
        let mut occluder = OcclusionShape2x2::default();
        for y in 0..2 {
            for x in 0..2 {
                if value[y][x] != 0 {
                    occluder.set(x, y, true);
                }
            }
        }
        OcclusionShape::S2x2(occluder)
    }
}

impl From<[[u8; 4]; 4]> for OcclusionShape {
    fn from(value: [[u8; 4]; 4]) -> Self {
        let mut occluder = OcclusionShape4x4::default();
        for y in 0..4 {
            for x in 0..4 {
                if value[y][x] != 0 {
                    occluder.set(x, y, true);
                }
            }
        }
        OcclusionShape::S4x4(occluder)
    }
}

impl From<[[u8; 8]; 8]> for OcclusionShape {
    fn from(value: [[u8; 8]; 8]) -> Self {
        let mut occluder = OcclusionShape8x8::default();
        for y in 0..8 {
            for x in 0..8 {
                if value[y][x] != 0 {
                    occluder.set(x, y, true);
                }
            }
        }
        OcclusionShape::S8x8(occluder)
    }
}

impl From<[[u8; 16]; 16]> for OcclusionShape {
    fn from(value: [[u8; 16]; 16]) -> Self {
        let mut occluder = OcclusionShape16x16::default();
        for y in 0..16 {
            for x in 0..16 {
                if value[y][x] != 0 {
                    occluder.set(x, y, true);
                }
            }
        }
        OcclusionShape::S16x16(occluder)
    }
}

impl From<OcclusionShape2x2> for OcclusionShape {
    fn from(value: OcclusionShape2x2) -> Self {
        OcclusionShape::S2x2(value)
    }
}

impl From<OcclusionShape4x4> for OcclusionShape {
    fn from(value: OcclusionShape4x4) -> Self {
        OcclusionShape::S4x4(value)
    }
}

impl From<OcclusionShape8x8> for OcclusionShape {
    fn from(value: OcclusionShape8x8) -> Self {
        OcclusionShape::S8x8(value)
    }
}

impl From<OcclusionShape16x16> for OcclusionShape {
    fn from(value: OcclusionShape16x16) -> Self {
        OcclusionShape::S16x16(value)
    }
}

impl From<OcclusionRect> for OcclusionShape {
    fn from(value: OcclusionRect) -> Self {
        OcclusionShape::Rect(value)
    }
}

#[test]
fn occlusion_test() {
    let occluder_a = OcclusionShape::from([
        [1,1,0,0],
        [1,1,0,0],
        [1,1,0,0],
        [1,1,0,0]
    ]);
    let occluder_b = OcclusionShape::from([
        [1,0],
        [1,0],
    ]);
    if occluder_a.occluded_by(&occluder_b) {
        println!("A Occluded by B");
    }
    if occluder_b.occluded_by(&occluder_a) {
        println!("B Occluded by A");
    }
}

#[test]
fn reduce_rect_test() {
    fn reduce_min(value: i32, base: i32, reduction: i32) -> i32 {
        value / (base / reduction)
    }
    fn reduce_max(value: i32, base: i32, reduction: i32) -> i32 {
        let div = base / reduction;
        value / div + if value % div == 0 { 0 } else { 1 }
    }
    let base = 16;
    let reduction = 8;
    let (left, right) = (2, 14);
    let (rl, rr) = (
        reduce_min(left, base, reduction),
        reduce_max(right, base, reduction)
    );
    println!("{rl}, {rr}");
}