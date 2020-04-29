//! This entire module only exists because of [https://www.gamedevs.org/uploads/fast-extraction-viewing-frustum-planes-from-world-view-projection-matrix.pdf]
//! and contains basically zero original work

use nalgebra_glm::{Mat4, Vec3};

#[derive(Clone, Copy)]
pub struct Sphere {
    pub location: Vec3,
    pub radius: f32,
}

#[derive(Clone, Copy)]
pub struct Plane {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
}

impl Plane {
    pub const fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self { a, b, c, d }
    }

    pub fn normalize(mut self) -> Self {
        let mag = (self.a * self.a + self.b * self.b + self.c * self.c).sqrt();

        self.a /= mag;
        self.b /= mag;
        self.c /= mag;
        self.d /= mag;

        self
    }

    pub fn distance(&self, point: Vec3) -> f32 {
        self.a * point.x + self.b * point.y + self.c * point.z + self.d
    }
}

#[derive(Clone, Copy)]
pub struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    pub fn from_matrix(matrix: Mat4) -> Self {
        let left = Plane {
            a: matrix.m41 + matrix.m11,
            b: matrix.m42 + matrix.m12,
            c: matrix.m43 + matrix.m13,
            d: matrix.m44 + matrix.m14,
        };

        let right = Plane {
            a: matrix.m41 - matrix.m11,
            b: matrix.m42 - matrix.m12,
            c: matrix.m43 - matrix.m13,
            d: matrix.m44 - matrix.m14,
        };

        let top = Plane {
            a: matrix.m41 - matrix.m21,
            b: matrix.m42 - matrix.m22,
            c: matrix.m43 - matrix.m23,
            d: matrix.m44 - matrix.m24,
        };

        let bottom = Plane {
            a: matrix.m41 + matrix.m21,
            b: matrix.m42 + matrix.m22,
            c: matrix.m43 + matrix.m23,
            d: matrix.m44 + matrix.m24,
        };

        let near = Plane {
            a: matrix.m31,
            b: matrix.m32,
            c: matrix.m33,
            d: matrix.m34,
        };

        let far = Plane {
            a: matrix.m41 - matrix.m31,
            b: matrix.m42 - matrix.m32,
            c: matrix.m43 - matrix.m33,
            d: matrix.m44 - matrix.m34,
        };

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
