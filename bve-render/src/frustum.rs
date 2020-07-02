//! This entire module only exists because of <https://www.gamedevs.org/uploads/fast-extraction-viewing-frustum-planes-from-world-view-projection-matrix.pdf/>
//! and contains basically zero original work

use glam::{Mat4, Vec3A};

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub location: Vec3A,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub abc: Vec3A,
    pub d: f32,
}

impl Plane {
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self {
            abc: Vec3A::new(a, b, c),
            d,
        }
    }

    pub fn normalize(mut self) -> Self {
        let mag = self.abc.length();

        self.abc /= mag;
        self.d /= mag;

        self
    }

    pub fn distance(&self, point: Vec3A) -> f32 {
        self.abc.dot(point) + self.d
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    pub planes: [Plane; 5],
}

impl Frustum {
    pub fn from_matrix(matrix: Mat4) -> Self {
        let mat_arr = matrix.to_cols_array_2d();

        let left = Plane::new(
            mat_arr[0][3] + mat_arr[0][0],
            mat_arr[1][3] + mat_arr[1][0],
            mat_arr[2][3] + mat_arr[2][0],
            mat_arr[3][3] + mat_arr[3][0],
        );

        let right = Plane::new(
            mat_arr[0][3] - mat_arr[0][0],
            mat_arr[1][3] - mat_arr[1][0],
            mat_arr[2][3] - mat_arr[2][0],
            mat_arr[3][3] - mat_arr[3][0],
        );

        let top = Plane::new(
            mat_arr[0][3] - mat_arr[0][1],
            mat_arr[1][3] - mat_arr[1][1],
            mat_arr[2][3] - mat_arr[2][1],
            mat_arr[3][3] - mat_arr[3][1],
        );

        let bottom = Plane::new(
            mat_arr[0][3] + mat_arr[0][1],
            mat_arr[1][3] + mat_arr[1][1],
            mat_arr[2][3] + mat_arr[2][1],
            mat_arr[3][3] + mat_arr[3][1],
        );

        // no far plane as we have infinite depth

        // this is the far plane in the algorithm, but we're using inverse Z, so near and far
        // get flipped.
        let near = Plane::new(
            mat_arr[0][3] - mat_arr[0][2],
            mat_arr[1][3] - mat_arr[1][2],
            mat_arr[2][3] - mat_arr[2][2],
            mat_arr[3][3] - mat_arr[3][2],
        );

        Self {
            planes: [
                left.normalize(),
                right.normalize(),
                top.normalize(),
                bottom.normalize(),
                near.normalize(),
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
