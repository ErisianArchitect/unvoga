#![allow(unused)]

use bevy::math::{*};

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

pub fn raycast<F: FnMut(IVec3, u32) -> bool>(
    ray_origin: Vec3,
    ray_direction: Vec3,
    cell_size: Vec3,
    cell_offset: Vec3,
    step_count: u32,
    mut callback: F,
) {
    fn calc_step(cell_size: f32, magnitude: f32) -> f32 {
        cell_size / magnitude.abs().max(<f32>::MIN_POSITIVE)
    }
    let delta = vec3(
        calc_step(cell_size.x, ray_direction.x),
        calc_step(cell_size.y, ray_direction.y),
        calc_step(cell_size.z, ray_direction.z),
    );

    let sign = ray_direction.signum();

    let step = ivec3(
        sign.x as i32,
        sign.y as i32,
        sign.z as i32,
    );

    let origin = ray_origin - cell_offset;
    let inner = origin.rem_euclid(cell_size);

    fn calc_t_max(step: i32, cell_size: f32, p: f32, magnitude: f32) -> f32 {
        if step > 0 {
            (cell_size - p) / magnitude.abs().max(<f32>::MIN_POSITIVE)
        } else if step < 0 {
            p / magnitude.abs().max(<f32>::MIN_POSITIVE)
        } else {
            f32::INFINITY
        }
    }
    let mut t_max = vec3(
        calc_t_max(step.x, cell_size.x, inner.x, ray_direction.x),
        calc_t_max(step.y, cell_size.y, inner.y, ray_direction.y),
        calc_t_max(step.z, cell_size.z, inner.z, ray_direction.z),
    );
 
    let mut cell = (origin / cell_size).floor().as_ivec3();
    callback(cell, 0);
    for step_index in 1..step_count {
        if t_max.x <= t_max.y && t_max.x <= t_max.z {
            t_max.x += delta.x;
            cell.x += step.x;
        } else if t_max.y <= t_max.z {
            t_max.y += delta.y;
            cell.y += step.y;
        } else {
            t_max.z += delta.z;
            cell.z += step.z;
        }
        // if t_max.x <= t_max.y {
        //     if t_max.x <= t_max.z {
        //         t_max.x += delta.x;
        //         cell.x += step.x;
        //     } else {
        //         t_max.z += delta.z;
        //         cell.z += step.z;
        //     }
        // } else {
        //     if t_max.y <= t_max.z {
        //         t_max.y += delta.y;
        //         cell.y += step.y;
        //     } else {
        //         t_max.z += delta.z;
        //         cell.z += step.z;
        //     }
        // }
        callback(cell, step_index);
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