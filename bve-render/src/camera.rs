use crate::*;
use glam::{Mat3, Vec3};

pub const NEAR_PLANE_DISTANCE: f32 = 0.1;

// Not used by the camera, as we have infinite depth, but by other
// frustum culling routines that need non-infinite depth
pub const FAR_PLANE_DISTANCE: f32 = 33.0 /* blocks */ * 64.0 /* m */;

pub struct Camera {
    pub location: Vec3,
    /// radians
    pub pitch: f32,
    /// radians
    pub yaw: f32,
}

impl Camera {
    pub fn compute_look_offset(&self) -> Vec3 {
        let starting = Vec3::unit_z();
        let post_pitch = Mat3::from_rotation_x(self.pitch) * starting;
        Mat3::from_rotation_y(self.yaw) * post_pitch
    }

    pub fn compute_matrix(&self) -> Mat4 {
        let look_offset = self.compute_look_offset();

        Mat4::look_at_lh(self.location, self.location + look_offset, Vec3::unit_y())
    }

    pub fn compute_origin_matrix(&self) -> Mat4 {
        let look_offset = self.compute_look_offset();

        Mat4::look_at_lh(Vec3::zero(), look_offset, Vec3::unit_y())
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
