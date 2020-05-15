pub use as_u32::*;
pub use hex::*;

mod as_u32;
mod hex;

use glam::{Vec2, Vec3, Vec4};

macro_rules! gen_uivec2 {
    ($($name:ident => $ty:ty),*) => {$(
        #[repr(C)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub struct $name {
            pub x: $ty,
            pub y: $ty,
        }

        impl $name {
            #[must_use]
            pub const fn new(x: $ty, y: $ty) -> Self {
                Self { x, y }
            }

            #[must_use]
            pub const fn splat(v: $ty) -> Self {
                Self { x: v, y: v }
            }

            #[must_use]
            pub const fn into_array(self) -> [$ty; 2] {
                [self.x, self.y]
            }

            #[must_use]
            pub const fn from_array([x, y]: [$ty; 2]) -> Self {
                Self { x, y }
            }

            pub fn map(self, mut f: impl FnMut($ty) -> $ty) -> Self {
                Self {
                    x: f(self.x),
                    y: f(self.y),
                }
            }

            pub fn map_f32(self, mut f: impl FnMut($ty) -> f32) -> Vec2 {
                Vec2::new(
                    f(self.x),
                    f(self.y)
                )
            }

            pub fn map_i32(self, mut f: impl FnMut($ty) -> i32) -> IVec2 {
                IVec2 {
                    x: f(self.x),
                    y: f(self.y),
                }
            }

            pub fn map_u32(self, mut f: impl FnMut($ty) -> u32) -> UVec2 {
                UVec2 {
                    x: f(self.x),
                    y: f(self.y),
                }
            }

            pub fn map_bool(self, mut f: impl FnMut($ty) -> bool) -> BVec2 {
                BVec2 {
                    x: f(self.x),
                    y: f(self.y),
                }
            }
        }

        impl From<[$ty; 2]> for $name {
            fn from(arr: [$ty; 2]) -> Self {
                Self::from_array(arr)
            }
        }

        impl From<$name> for [$ty; 2] {
            fn from(vec: $name) -> Self {
                vec.into_array()
            }
        }
    )*};
}
macro_rules! gen_uivec3 {
    ($($name:ident => $ty:ty),*) => {$(
        #[repr(C)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub struct $name {
            pub x: $ty,
            pub y: $ty,
            pub z: $ty,
        }

        impl $name {
            #[must_use]
            pub const fn new(x: $ty, y: $ty, z: $ty) -> Self {
                Self { x, y, z }
            }

            #[must_use]
            pub const fn splat(v: $ty) -> Self {
                Self { x: v, y: v, z: v }
            }

            #[must_use]
            pub const fn into_array(self) -> [$ty; 3] {
                [self.x, self.y, self.z]
            }

            #[must_use]
            pub const fn from_array([x, y, z]: [$ty; 3]) -> Self {
                Self { x, y, z }
            }

            pub fn map(self, mut f: impl FnMut($ty) -> $ty) -> Self {
                Self {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                }
            }

            pub fn map_f32(self, mut f: impl FnMut($ty) -> f32) -> Vec3 {
                Vec3::new(
                    f(self.x),
                    f(self.y),
                    f(self.z)
                )
            }

            pub fn map_i32(self, mut f: impl FnMut($ty) -> i32) -> IVec3 {
                IVec3 {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                }
            }

            pub fn map_u32(self, mut f: impl FnMut($ty) -> u32) -> UVec3 {
                UVec3 {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                }
            }

            pub fn map_color(self, mut f: impl FnMut($ty) -> u8) -> ColorU8RGB {
                ColorU8RGB {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                }
            }

            pub fn map_bool(self, mut f: impl FnMut($ty) -> bool) -> BVec3 {
                BVec3 {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.y),
                }
            }
        }

        impl From<[$ty; 3]> for $name {
            fn from(arr: [$ty; 3]) -> Self {
                Self::from_array(arr)
            }
        }

        impl From<$name> for [$ty; 3] {
            fn from(vec: $name) -> Self {
                vec.into_array()
            }
        }
    )*};
}
macro_rules! gen_uivec4 {
    ($($name:ident => $ty:ty),*) => {$(
        #[repr(C)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub struct $name {
            pub x: $ty,
            pub y: $ty,
            pub z: $ty,
            pub w: $ty,
        }

        impl $name {
            #[must_use]
            pub const fn new(x: $ty, y: $ty, z: $ty, w: $ty) -> Self {
                Self { x, y, z, w }
            }

            #[must_use]
            pub const fn splat(v: $ty) -> Self {
                Self { x: v, y: v, z: v, w: v }
            }

            #[must_use]
            pub const fn into_array(self) -> [$ty; 4] {
                [self.x, self.y, self.z, self.w]
            }

            #[must_use]
            pub const fn from_array([x, y, z, w]: [$ty; 4]) -> Self {
                Self { x, y, z, w }
            }

            pub fn map(self, mut f: impl FnMut($ty) -> $ty) -> Self {
                Self {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                    w: f(self.w),
                }
            }

            pub fn map_f32(self, mut f: impl FnMut($ty) -> f32) -> Vec4 {
                Vec4::new(
                    f(self.x),
                    f(self.y),
                    f(self.z),
                    f(self.w)
                )
            }

            pub fn map_i32(self, mut f: impl FnMut($ty) -> i32) -> IVec4 {
                IVec4 {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                    w: f(self.w),
                }
            }

            pub fn map_u32(self, mut f: impl FnMut($ty) -> u32) -> UVec4 {
                UVec4 {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                    w: f(self.w),
                }
            }

            pub fn map_color(self, mut f: impl FnMut($ty) -> u8) -> ColorU8RGBA {
                ColorU8RGBA {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                    w: f(self.w),
                }
            }

            pub fn map_bool(self, mut f: impl FnMut($ty) -> bool) -> BVec4 {
                BVec4 {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.y),
                    w: f(self.w),
                }
            }
        }

        impl From<[$ty; 4]> for $name {
            fn from(arr: [$ty; 4]) -> Self {
                Self::from_array(arr)
            }
        }

        impl From<$name> for [$ty; 4] {
            fn from(vec: $name) -> Self {
                vec.into_array()
            }
        }
    )*};
}

gen_uivec2!(BVec2 => bool, UVec2 => u32, IVec2 => i32);
gen_uivec3!(BVec3 => bool, ColorU8RGB => u8, UVec3 => u32, IVec3 => i32);
gen_uivec4!(BVec4 => bool, ColorU8RGBA => u8, UVec4 => u32, IVec4 => i32);
