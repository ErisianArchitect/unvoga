use bevy::{math::{Quat, Vec2, Vec3}, prelude::{Mut, Resource, Transform}};

pub enum CameraType {
    /// Elevation stays locked when moving forward, backward, left or right.
    Pan,
    /// Elevation changes when moving forward or backward while looking upward or downward.
    Free,
}

pub struct CameraContoller {
    pub cam_type: CameraType,
    pub move_speed: f32,
    pub rotate_speed: f32,
    pub rotation: Vec2,
}

impl CameraContoller {
    pub fn new(cam_type: CameraType, rotation: Vec2, move_speed: f32, rotate_speed: f32) -> Self {
        Self {
            cam_type,
            move_speed,
            rotate_speed,
            rotation,
        }
    }

    pub fn begin_transform<'a>(&'a mut self, transform: Mut<'a, Transform>) -> CameraTransformController<'a> {
        CameraTransformController {
            transform,
            controller: self
        }
    }
}

pub struct CameraTransformController<'a> {
    pub transform: Mut<'a, Transform>,
    pub controller: &'a mut CameraContoller,
}

impl<'a> CameraTransformController<'a> {
    pub fn end_transform(self) -> Mut<'a, Transform> {
        self.transform
    }

    pub fn rotate(&mut self, rotation: Vec2, delta_time: f32, multiplier: f32) {
        let dts = delta_time * self.controller.rotate_speed * multiplier;
        self.controller.rotation.x += rotation.x * dts;
        self.controller.rotation.y += rotation.y * dts;
        self.controller.rotation.y = self.controller.rotation.y.clamp(-90f32.to_radians(), 90f32.to_radians());
        self.transform.rotation = Quat::from_axis_angle(Vec3::NEG_Y, self.controller.rotation.x) * Quat::from_axis_angle(Vec3::NEG_X, self.controller.rotation.y)
    }

    pub fn translate(&mut self, translation: Vec3, delta_time: f32, multiplier: f32) {
        let dts = delta_time * self.controller.move_speed * multiplier;
        match self.controller.cam_type {
            CameraType::Free => {
                let fwd = translation.z * dts;
                let up = translation.y * dts;
                let right = translation.x * dts;
                let mut translation = Vec3::ZERO;
                translation += -fwd * self.transform.forward().normalize();
                translation += up * self.transform.up().normalize();
                translation += right * self.transform.right().normalize();
                self.transform.translation += translation;
            }
            CameraType::Pan => {
                let rotater = Quat::from_axis_angle(Vec3::NEG_Y, self.controller.rotation.x);
                self.transform.translation += rotater * (translation * dts);
            }
        }
    }
}