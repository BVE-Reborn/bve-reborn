//! This entire module only exists because of <https://www.gamedevs.org/uploads/fast-extraction-viewing-frustum-planes-from-world-view-projection-matrix.pdf/>
//! and contains basically zero original work

use nalgebra_glm::{make_vec3, Mat4, Vec3};

#[derive(Clone, Copy)]
pub struct Sphere {
    pub location: Vec3,
    pub radius: f32,
}

#[derive(Clone, Copy)]
pub struct Plane {
    pub abc: Vec3,
    pub d: f32,
}

impl Plane {
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self {
            abc: make_vec3(&[a, b, c]),
            d,
        }
    }

    pub fn normalize(mut self) -> Self {
        let mag = self.abc.magnitude();

        self.abc /= mag;
        self.d /= mag;

        self
    }

    pub fn distance(&self, point: Vec3) -> f32 {
        self.abc.dot(&point) + self.d
    }
}

#[derive(Clone, Copy)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    pub fn from_matrix(matrix: Mat4) -> Self {
        let left = Plane::new(
            matrix.m41 + matrix.m11,
            matrix.m42 + matrix.m12,
            matrix.m43 + matrix.m13,
            matrix.m44 + matrix.m14,
        );

        let right = Plane::new(
            matrix.m41 - matrix.m11,
            matrix.m42 - matrix.m12,
            matrix.m43 - matrix.m13,
            matrix.m44 - matrix.m14,
        );

        let top = Plane::new(
            matrix.m41 - matrix.m21,
            matrix.m42 - matrix.m22,
            matrix.m43 - matrix.m23,
            matrix.m44 - matrix.m24,
        );

        let bottom = Plane::new(
            matrix.m41 + matrix.m21,
            matrix.m42 + matrix.m22,
            matrix.m43 + matrix.m23,
            matrix.m44 + matrix.m24,
        );

        let near = Plane::new(matrix.m31, matrix.m32, matrix.m33, matrix.m34);

        let far = Plane::new(
            matrix.m41 - matrix.m31,
            matrix.m42 - matrix.m32,
            matrix.m43 - matrix.m33,
            matrix.m44 - matrix.m34,
        );

        Self {
            planes: [
                left.normalize(),
                right.normalize(),
                top.normalize(),
                bottom.normalize(),
                near.normalize(),
                far.normalize(),
            ],
        }
    }

    pub fn contains_sphere(&self, sphere: Sphere) -> bool {
        // ref: https://wiki.ogre3d.org/Frustum+Culling+In+Object+Space
        // the normals of the planes point into the frustum, so the distance to a visible object right on the edge of
        // the frustum would be just greater than -radius
        self.planes
            .iter()
            .all(|plane| plane.distance(sphere.location) >= -sphere.radius)
    }
}
