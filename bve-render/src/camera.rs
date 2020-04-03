use crate::*;
use cgmath::{EuclideanSpace, Matrix3, Matrix4, Point3, Rad, SquareMatrix, Vector3, Vector4};

pub struct Camera {
    pub location: Vector3<f32>,
    /// radians
    pub pitch: f32,
    /// radians
    pub yaw: f32,
}

impl Camera {
    pub fn compute_matrix(&self) -> Matrix4<f32> {
        // This is pre z-inversion, so z is flipped here
        let look_offset = Matrix3::from_diagonal(Vector3::new(1.0, 1.0, -1.0))
            * Matrix3::from_axis_angle(Vector3::unit_y(), Rad(self.yaw))
            * Matrix3::from_axis_angle(Vector3::unit_x(), Rad(self.pitch))
            * Vector3::unit_z();

        Matrix4::from_diagonal(Vector4::new(1.0, 1.0, -1.0, 1.0))
            * Matrix4::look_at(
                Point3::from_vec(self.location),
                Point3::from_vec(self.location + look_offset),
                Vector3::unit_y(),
            )
    }
}

impl Renderer {
    pub fn set_camera_orientation(&mut self, pitch: f32, yaw: f32) {
        self.camera.pitch = pitch;
        self.camera.yaw = yaw;
    }

    pub fn set_camera_location(&mut self, location: Vector3<f32>) {
        self.camera.location = location;
    }
}
