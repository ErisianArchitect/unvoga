use core::f32;

use bevy::math::{vec3, Ray3d, Vec3};

use crate::prelude::{Direction, Orientation};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct AABB {
    min: Vec3,
    max: Vec3
}

impl AABB {
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min,
            max
        }
    }

    pub fn from_bounds(a: Vec3, b: Vec3) -> Self {
        Self {
            min: vec3(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            max: vec3(b.x.max(a.x), b.y.max(a.y), b.z.max(a.z)),
        }
    }

    #[inline]
    pub fn intersects(self, ray: Ray3d) -> Option<f32> {
        self.intersects_frac(ray, Self::calc_dirfrac(ray))
    }

    pub fn intersects_frac(self, ray: Ray3d, dirfrac: Vec3) -> Option<f32> {
        let t1 = (self.min.x - ray.origin.x) * dirfrac.x;
        let t2 = (self.max.x - ray.origin.x) * dirfrac.x;
        let t3 = (self.min.y - ray.origin.y) * dirfrac.y;
        let t4 = (self.max.y - ray.origin.y) * dirfrac.y;
        let t5 = (self.min.z - ray.origin.z) * dirfrac.z;
        let t6 = (self.max.z - ray.origin.z) * dirfrac.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));
        if tmax < 0.0 {
            None
        } else if tmin > tmax {
            None
        } else {
            Some(tmin)
        }
    }

    pub fn intersection_point(self, ray: Ray3d) -> Option<Vec3> {
        let dist = self.intersects(ray)?;
        Some(ray.origin + (ray.direction * dist))
    }

    /// Used for fast AABB intersection.
    /// When you need to intersect a lot of AABBs with the same ray,
    /// you can use this function to calculate a `dirfrac` Vec3 that
    /// can be passed to [AABB::intersects_frac] to calculate intersection.
    #[inline]
    pub fn calc_dirfrac(ray: Ray3d) -> Vec3 {
        vec3(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z
        )
    }

    pub fn intersects_box(self, other: AABB) -> bool {
        self.min.x < other.max.x &&
        self.min.y < other.max.y &&
        self.min.z < other.max.z &&
        other.min.x < self.max.x &&
        other.min.y < self.max.y &&
        other.min.z < self.max.z
    }

    pub fn contains(self, point: Vec3) -> bool {
        self.min.x <= point.x &&
        self.min.y <= point.y &&
        self.min.z <= point.z &&
        self.max.x > point.x &&
        self.max.y > point.y &&
        self.max.z > point.z
    }

    /// For when you want to apply an orientation while keeping the same relative center.
    pub fn orient_centered(self, orientation: Orientation) -> Self {
        let diff = self.max - self.min;
        let center = self.min + (diff * 0.5);
        let rel_min = self.min - center;
        let rel_max = self.max - center;
        let a = orientation.transform(rel_min) + center;
        let b = orientation.transform(rel_max) + center;
        Self::from_bounds(a, b)
    }
    
    /// This does not preserve the center! This will orient around the world origin (0, 0, 0).
    /// If you want the bounding box to remain centered in the same position, use orient_centered.
    pub fn orient(self, orientation: Orientation) -> Self {
        let a = orientation.transform(self.min);
        let b = orientation.transform(self.max);
        Self::from_bounds(a, b)
    }

    /// Orients inside of voxel space.
    /// This will apply the orientation and then translate into voxel space.
    pub fn orient_voxel<C: Into<(i32, i32, i32)>>(self, coord: C, orientation: Orientation) -> Self {
        self.orient(orientation).translate_voxel(coord)
    }

    /// Translate into voxel space.
    pub fn translate_voxel<C: Into<(i32, i32, i32)>>(self, coord: C) -> Self {
        let (x, y, z) = coord.into();
        let offset = vec3(
            x as f32 + 0.5,
            y as f32 + 0.5,
            z as f32 + 0.5
        );
        Self::new(self.min + offset, self.max + offset)
    }

    pub fn voxel<C: Into<(i32, i32, i32)>>(coord: C) -> Self {
        let (x, y, z) = coord.into();
        let min = vec3(
            x as f32,
            y as f32,
            z as f32,
        );
        let max = vec3(
            (x + 1) as f32,
            (y + 1) as f32,
            (z + 1) as f32
        );
        Self {
            min,
            max
        }
    }

    
}

#[inline(always)]
fn inv_d(d: f32) -> f32 {
    if d != 0.0 {
        1.0 / d
    } else {
        f32::INFINITY
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::{Flip, Rotation};

    use super::*;
    #[test]
    fn intersect_test() {
        let ray = Ray3d::new(vec3(-5.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0).normalize());
        
        let aabb = AABB::from_bounds(Vec3::splat(1.0) * -0.5, Vec3::splat(1.0) * 0.5);
        if let Some(point) = aabb.intersection_point(ray) {
            println!("Intersects at {point}");
            // let origin = ray.origin;
            // let direction = ray.direction;
            // println!("   Origin: {origin}");
            // // println!("Direction: {direction}");
            // println!(" Distance: {dist}");
            // println!("Magnitude: {}", direction * dist);
            // println!("Intersection at {} {dist}", (ray.origin + (ray.direction * dist)));
        } else {
            println!("No intersection");
        }
    }

    #[test]
    fn orientation_test() {
        let aabb = AABB::from_bounds(
            vec3(-0.5, -2.5, -0.5),
            vec3(0.5, 7.5, 0.5),
        );
        let orientation = Orientation::new(Rotation::new(Direction::PosX, 1), Flip::NONE);
        let ort = aabb.orient_centered(orientation);
        println!("{ort:?}");
    }
}