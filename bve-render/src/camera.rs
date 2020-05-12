use crate::*;
use nalgebra_glm::{look_at_lh, make_vec3, rotate_vec3, Vec3};

pub const NEAR_PLANE_DISTANCE: f32 = 0.1;
// 33 blocks * 64m
pub const FAR_PLANE_DISTANCE: f32 = 2112.0;

pub struct Camera {
    pub location: Vec3,
    /// radians
    pub pitch: f32,
    /// radians
    pub yaw: f32,
}

impl Camera {
    pub fn compute_look_offset(&self) -> Vec3 {
        let starting = make_vec3(&[0.0, 0.0, 1.0]);
        let post_pitch = rotate_vec3(&starting, self.pitch, &make_vec3(&[1.0, 0.0, 0.0]));
        rotate_vec3(&post_pitch, self.yaw, &make_vec3(&[0.0, 1.0, 0.0]))
    }

    pub fn compute_matrix(&self) -> Mat4 {
        let look_offset = self.compute_look_offset();

        look_at_lh(
            &self.location,
            &(self.location + look_offset),
            &make_vec3(&[0.0, 1.0, 0.0]),
        )
    }

    pub fn compute_origin_matrix(&self) -> Mat4 {
        let look_offset = self.compute_look_offset();

        look_at_lh(&make_vec3(&[0.0, 0.0, 0.0]), &look_offset, &make_vec3(&[0.0, 1.0, 0.0]))
    }
}

impl Renderer {
    pub fn set_camera_orientation(&mut self, pitch: f32, yaw: f32) {
        self.camera.pitch = pitch;
        self.camera.yaw = yaw;
    }

    pub fn set_camera_location(&mut self, location: Vec3) {
        self.camera.location = location;
    }
}
