#![allow(unused)]

use bevy::math::{Vec2, Vec3, Vec4};

pub fn index2<const W: i32>(x: i32, y: i32) -> usize {
    let x = x.rem_euclid(W);
    let y = y.rem_euclid(W);
    (y * W + x) as usize
}


pub fn index3<const W: i32>(x: i32, y: i32, z: i32) -> usize {
    let x = x.rem_euclid(W);
    let y = y.rem_euclid(W);
    let z = z.rem_euclid(W);
    (y * W*W + z * W + x) as usize
}

/// Returns (min, max)
pub fn minmax<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a <= b { (a, b) } else { (b, a) }
}

pub fn f32_not_zero(value: f32) -> bool {
    value != 0.0 && value != -0.0
}

pub fn f32_is_zero(value: f32) -> bool {
    value == 0.0 || value == -0.0
}

pub fn f64_not_zero(value: f64) -> bool {
    value != 0.0 && value != -0.0
}

pub fn f64_is_zero(value: f64) -> bool {
    value == 0.0 || value == -0.0
}

/// Returns `Some(t)` where t is the normalized distance between the min and max.
/// So if the min and max were 5 and 10 and you wanted to check the value of
/// 7.5, you would expect to get a result of `Some(0.5)` because 7.5 is halfway
/// between 5 and 10.
pub fn check_between_f32(value: f32, min: f32, max: f32) -> Option<f32> {
    if value < min || value > max {
        None
    } else {
        let diff = max - min;
        let mult = 1.0 / diff;
        let value_in = value - min;
        Some(value_in * mult)
    }
}

/// Returns `Some(t)` where t is the normalized distance between the min and max.
/// So if the min and max were 5 and 10 and you wanted to check the value of
/// 7.5, you would expect to get a result of `Some(0.5)` because 7.5 is halfway
/// between 5 and 10.
pub fn check_between_f64(value: f64, min: f64, max: f64) -> Option<f64> {
    if value < min || value > max {
        None
    } else {
        let diff = max - min;
        let mult = 1.0 / diff;
        let value_in = value - min;
        Some(value_in * mult)
    }
}

pub fn check_between_vec2(value: Vec2, min: Vec2, max: Vec2) -> Option<f32> {
    let ab = max - min;
    let ap = value - min;
    let ab_dot_ab = ab.dot(ab);
    let ap_dot_ab = ap.dot(ab);
    let t = ap_dot_ab / ab_dot_ab;
    if 0.0 <= t && t <= 1.0 {
        Some(t)
    } else {
        None
    }
}

pub fn check_between_vec3(value: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let ab = max - min;
    let ap = value - min;
    let ab_dot_ab = ab.dot(ab);
    let ap_dot_ab = ap.dot(ab);
    let t = ap_dot_ab / ab_dot_ab;
    if 0.0 <= t && t <= 1.0 {
        Some(t)
    } else {
        None
    }
}

pub fn check_between_vec4(value: Vec4, min: Vec4, max: Vec4) -> Option<f32> {
    let ab = max - min;
    let ap = value - min;
    let ab_dot_ab = ab.dot(ab);
    let ap_dot_ab = ap.dot(ab);
    let t = ap_dot_ab / ab_dot_ab;
    if 0.0 <= t && t <= 1.0 {
        Some(t)
    } else {
        None
    }
}

pub fn check_between_vec2_closest(value: Vec2, min: Vec2, max: Vec2) -> Option<Vec2> {
    let ab = max - min;
    let ap = value - min;
    let ab_dot_ab = ab.dot(ab);
    let ap_dot_ab = ap.dot(ab);
    let t = ap_dot_ab / ab_dot_ab;
    if 0.0 <= t && t <= 1.0 {
        Some(min + t * ab)
    } else {
        None
    }
}

pub fn check_between_vec3_closest(value: Vec3, min: Vec3, max: Vec3) -> Option<Vec3> {
    let ab = max - min;
    let ap = value - min;
    let ab_dot_ab = ab.dot(ab);
    let ap_dot_ab = ap.dot(ab);
    let t = ap_dot_ab / ab_dot_ab;
    if 0.0 <= t && t <= 1.0 {
        Some(min + t * ab)
    } else {
        None
    }
}

pub fn check_between_vec4_closest(value: Vec4, min: Vec4, max: Vec4) -> Option<Vec4> {
    let ab = max - min;
    let ap = value - min;
    let ab_dot_ab = ab.dot(ab);
    let ap_dot_ab = ap.dot(ab);
    let t = ap_dot_ab / ab_dot_ab;
    if 0.0 <= t && t <= 1.0 {
        Some(min + t * ab)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_between_test() {
        if let Some(between) = check_between_f32(7.5, 5.0, 10.0) {
            println!("Between: {between}");
        } else {
            panic!("Not between???");
        }
    }
}

pub trait CheckBetween {
    type Output;
    fn check_between(self, min: Self, max: Self) -> Option<Self::Output>;
}

pub trait CheckBetweenClosest: Sized {
    fn check_between_closest(self, min: Self, max: Self) -> Option<Self>;
}

impl CheckBetween for f32 {
    type Output = f32;
    fn check_between(self, min: Self, max: Self) -> Option<f32> {
        check_between_f32(self, min, max)
    }
}

impl CheckBetween for f64 {
    type Output = f64;
    fn check_between(self, min: Self, max: Self) -> Option<f64> {
        check_between_f64(self, min, max)
    }
}

impl CheckBetween for Vec2 {
    type Output = f32;
    fn check_between(self, min: Self, max: Self) -> Option<f32> {
        check_between_vec2(self, min, max)
    }
}

impl CheckBetween for Vec3 {
    type Output = f32;
    fn check_between(self, min: Self, max: Self) -> Option<f32> {
        check_between_vec3(self, min, max)
    }
}

impl CheckBetween for Vec4 {
    type Output = f32;
    fn check_between(self, min: Self, max: Self) -> Option<f32> {
        check_between_vec4(self, min, max)
    }
}

impl CheckBetweenClosest for Vec2 {
    fn check_between_closest(self, min: Self, max: Self) -> Option<Self> {
        check_between_vec2_closest(self, min, max)
    }
}

impl CheckBetweenClosest for Vec3 {
    fn check_between_closest(self, min: Self, max: Self) -> Option<Self> {
        check_between_vec3_closest(self, min, max)
    }
}

impl CheckBetweenClosest for Vec4 {
    fn check_between_closest(self, min: Self, max: Self) -> Option<Self> {
        check_between_vec4_closest(self, min, max)
    }
}

#[cfg(test)]
mod testing_sandbox {
    use bevy::math::vec2;

    // TODO: Remove this sandbox when it is no longer in use.
    use super::*;
    #[test]
    fn sandbox() {
        let a = vec2(0.0, 0.0);
        let b = vec2(1.0, 1.0);
        let p = vec2(0.3, 0.5);
        if let Some(closest) = p.check_between_closest(a, b) {
            println!("Closest: {closest}");
        }
    }
}