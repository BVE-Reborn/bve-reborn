use glam::{Vec2, Vec3A, Vec4};

macro_rules! gen_uivec2 {
    ($($name:ident => $ty:ty),*) => {$(
        #[repr(C)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub struct $name {
            pub x: $ty,
            pub y: $ty,
        }

        #[allow(clippy::use_self)]
        impl $name {
            #[must_use]
            #[inline]
            pub const fn new(x: $ty, y: $ty) -> Self {
                Self { x, y }
            }

            #[must_use]
            #[inline]
            pub const fn splat(v: $ty) -> Self {
                Self { x: v, y: v }
            }

            #[must_use]
            #[inline]
            pub const fn into_array(self) -> [$ty; 2] {
                [self.x, self.y]
            }

            #[must_use]
            #[inline]
            pub const fn from_array([x, y]: [$ty; 2]) -> Self {
                Self { x, y }
            }

            #[inline]
            pub fn map(self, mut f: impl FnMut($ty) -> $ty) -> Self {
                Self {
                    x: f(self.x),
                    y: f(self.y),
                }
            }

            #[inline]

            pub fn map_f32(self, mut f: impl FnMut($ty) -> f32) -> Vec2 {
                Vec2::new(
                    f(self.x),
                    f(self.y)
                )
            }

            #[inline]
            pub fn map_i32(self, mut f: impl FnMut($ty) -> i32) -> IVec2 {
                IVec2 {
                    x: f(self.x),
                    y: f(self.y),
                }
            }

            #[inline]
            pub fn map_u32(self, mut f: impl FnMut($ty) -> u32) -> UVec2 {
                UVec2 {
                    x: f(self.x),
                    y: f(self.y),
                }
            }

            #[inline]
            pub fn map_bool(self, mut f: impl FnMut($ty) -> bool) -> BVec2 {
                BVec2 {
                    x: f(self.x),
                    y: f(self.y),
                }
            }
        }

        impl From<[$ty; 2]> for $name {
            #[inline]
            fn from(arr: [$ty; 2]) -> Self {
                Self::from_array(arr)
            }
        }

        impl From<$name> for [$ty; 2] {
            #[inline]
            fn from(vec: $name) -> Self {
                vec.into_array()
            }
        }
    )*};
}
macro_rules! numeric_uivec2 {
    ($($name:ident => $ty:ty),*) => {
    gen_uivec2!($($name => $ty),*);
    $(
        impl $name {
            #[must_use]
            #[inline]
            pub const fn zero() -> Self {
                Self { x: 0, y: 0 }
            }

            #[must_use]
            #[inline]
            pub const fn one() -> Self {
                Self { x: 1, y: 1 }
            }
        }

        impl std::ops::Add<$name> for $name {
            type Output = Self;
            #[inline]
            fn add(self, other: Self) -> Self::Output {
                $name::new(self.x + other.x, self.y + other.y)
            }
        }

        impl std::ops::AddAssign<$name> for $name {
            #[inline]
            fn add_assign(&mut self, other: Self) {
                *self = *self + other;
            }
        }

        impl std::ops::Sub<$name> for $name {
            type Output = Self;
            #[inline]
            fn sub(self, other: Self) -> Self::Output {
                $name::new(self.x - other.x, self.y - other.y)
            }
        }

        impl std::ops::SubAssign<$name> for $name {
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                *self = *self - other;
            }
        }

        impl std::ops::Mul<$name> for $name {
            type Output = Self;
            #[inline]
            fn mul(self, other: Self) -> Self::Output {
                $name::new(self.x * other.x, self.y * other.y)
            }
        }

        impl std::ops::MulAssign<$name> for $name {
            #[inline]
            fn mul_assign(&mut self, other: Self) {
                *self = *self * other;
            }
        }

        impl std::ops::Div<$name> for $name {
            type Output = Self;
            #[inline]
            fn div(self, other: Self) -> Self::Output {
                $name::new(self.x / other.x, self.y / other.y)
            }
        }

        impl std::ops::DivAssign<$name> for $name {
            #[inline]
            fn div_assign(&mut self, other: Self) {
                *self = *self / other;
            }
        }

        #[allow(clippy::modulo_arithmetic)]
        impl std::ops::Rem<$name> for $name {
            type Output = Self;
            #[inline]
            fn rem(self, other: Self) -> Self::Output {
                $name::new(self.x % other.x, self.y % other.y)
            }
        }

        impl std::ops::RemAssign<$name> for $name {
            #[inline]
            fn rem_assign(&mut self, other: Self) {
                *self = *self % other;
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

        #[allow(clippy::use_self)]
        impl $name {
            #[must_use]
            #[inline]
            pub const fn new(x: $ty, y: $ty, z: $ty) -> Self {
                Self { x, y, z }
            }

            #[must_use]
            #[inline]
            pub const fn splat(v: $ty) -> Self {
                Self { x: v, y: v, z: v }
            }

            #[must_use]
            #[inline]
            pub const fn into_array(self) -> [$ty; 3] {
                [self.x, self.y, self.z]
            }

            #[must_use]
            #[inline]
            pub const fn from_array([x, y, z]: [$ty; 3]) -> Self {
                Self { x, y, z }
            }

            #[inline]
            pub fn map(self, mut f: impl FnMut($ty) -> $ty) -> Self {
                Self {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                }
            }

            #[inline]
            pub fn map_f32(self, mut f: impl FnMut($ty) -> f32) -> Vec3A {
                Vec3A::new(
                    f(self.x),
                    f(self.y),
                    f(self.z)
                )
            }

            #[inline]
            pub fn map_i32(self, mut f: impl FnMut($ty) -> i32) -> IVec3A {
                IVec3A {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                }
            }

            #[inline]
            pub fn map_u32(self, mut f: impl FnMut($ty) -> u32) -> UVec3A {
                UVec3A {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                }
            }

            #[inline]
            pub fn map_color(self, mut f: impl FnMut($ty) -> u8) -> ColorU8RGB {
                ColorU8RGB {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                }
            }

            #[inline]
            pub fn map_bool(self, mut f: impl FnMut($ty) -> bool) -> BVec3A {
                BVec3A {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.y),
                }
            }
        }

        impl From<[$ty; 3]> for $name {
            #[inline]
            fn from(arr: [$ty; 3]) -> Self {
                Self::from_array(arr)
            }
        }

        impl From<$name> for [$ty; 3] {
            #[inline]
            fn from(vec: $name) -> Self {
                vec.into_array()
            }
        }
    )*};
}
macro_rules! numeric_uivec3 {
    ($($name:ident => $ty:ty),*) => {
    gen_uivec3!($($name => $ty),*);
    $(
        impl $name {
            #[must_use]
            #[inline]
            pub const fn zero() -> Self {
                Self { x: 0, y: 0, z: 0 }
            }

            #[must_use]
            #[inline]
            pub const fn one() -> Self {
                Self { x: 1, y: 1, z: 1 }
            }
        }

        impl std::ops::Add<$name> for $name {
            type Output = Self;
            #[inline]
            fn add(self, other: Self) -> Self::Output {
                $name::new(self.x + other.x, self.y + other.y, self.z + other.z)
            }
        }

        impl std::ops::AddAssign<$name> for $name {
            #[inline]
            fn add_assign(&mut self, other: Self) {
                *self = *self + other;
            }
        }

        impl std::ops::Sub<$name> for $name {
            type Output = Self;
            #[inline]
            fn sub(self, other: Self) -> Self::Output {
                $name::new(self.x - other.x, self.y - other.y, self.z - other.z)
            }
        }

        impl std::ops::SubAssign<$name> for $name {
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                *self = *self - other;
            }
        }

        impl std::ops::Mul<$name> for $name {
            type Output = Self;
            #[inline]
            fn mul(self, other: Self) -> Self::Output {
                $name::new(self.x * other.x, self.y * other.y, self.z * other.z)
            }
        }

        impl std::ops::MulAssign<$name> for $name {
            #[inline]
            fn mul_assign(&mut self, other: Self) {
                *self = *self * other;
            }
        }

        impl std::ops::Div<$name> for $name {
            type Output = Self;
            #[inline]
            fn div(self, other: Self) -> Self::Output {
                $name::new(self.x / other.x, self.y / other.y, self.z / other.z)
            }
        }

        impl std::ops::DivAssign<$name> for $name {
            #[inline]
            fn div_assign(&mut self, other: Self) {
                *self = *self / other;
            }
        }

        #[allow(clippy::modulo_arithmetic)]
        impl std::ops::Rem<$name> for $name {
            type Output = Self;
            #[inline]
            fn rem(self, other: Self) -> Self::Output {
                $name::new(self.x % other.x, self.y % other.y, self.z % other.z)
            }
        }

        impl std::ops::RemAssign<$name> for $name {
            #[inline]
            fn rem_assign(&mut self, other: Self) {
                *self = *self % other;
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

        #[allow(clippy::use_self)]
        impl $name {
            #[must_use]
            #[inline]
            pub const fn new(x: $ty, y: $ty, z: $ty, w: $ty) -> Self {
                Self { x, y, z, w }
            }

            #[must_use]
            #[inline]
            pub const fn splat(v: $ty) -> Self {
                Self { x: v, y: v, z: v, w: v }
            }

            #[must_use]
            #[inline]
            pub const fn into_array(self) -> [$ty; 4] {
                [self.x, self.y, self.z, self.w]
            }

            #[must_use]
            #[inline]
            pub const fn from_array([x, y, z, w]: [$ty; 4]) -> Self {
                Self { x, y, z, w }
            }

            #[inline]
            pub fn map(self, mut f: impl FnMut($ty) -> $ty) -> Self {
                Self {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                    w: f(self.w),
                }
            }

            #[inline]

            pub fn map_f32(self, mut f: impl FnMut($ty) -> f32) -> Vec4 {
                Vec4::new(
                    f(self.x),
                    f(self.y),
                    f(self.z),
                    f(self.w)
                )
            }

            #[inline]
            pub fn map_i32(self, mut f: impl FnMut($ty) -> i32) -> IVec4 {
                IVec4 {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                    w: f(self.w),
                }
            }

            #[inline]
            pub fn map_u32(self, mut f: impl FnMut($ty) -> u32) -> UVec4 {
                UVec4 {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                    w: f(self.w),
                }
            }

            #[inline]
            pub fn map_color(self, mut f: impl FnMut($ty) -> u8) -> ColorU8RGBA {
                ColorU8RGBA {
                    x: f(self.x),
                    y: f(self.y),
                    z: f(self.z),
                    w: f(self.w),
                }
            }

            #[inline]
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
            #[inline]
            fn from(arr: [$ty; 4]) -> Self {
                Self::from_array(arr)
            }
        }

        impl From<$name> for [$ty; 4] {
            #[inline]
            fn from(vec: $name) -> Self {
                vec.into_array()
            }
        }
    )*};
}
macro_rules! numeric_uivec4 {
    ($($name:ident => $ty:ty),*) => {
    gen_uivec4!($($name => $ty),*);
    $(
        impl $name {
            #[must_use]
            #[inline]
            pub const fn zero() -> Self {
                Self { x: 0, y: 0, z: 0, w: 0 }
            }

            #[must_use]
            #[inline]
            pub const fn one() -> Self {
                Self { x: 1, y: 1, z: 1, w: 0 }
            }
        }

        impl std::ops::Add<$name> for $name {
            type Output = Self;
            #[inline]
            fn add(self, other: Self) -> Self::Output {
                $name::new(self.x + other.x, self.y + other.y, self.z + other.z, self.w + other.w)
            }
        }

        impl std::ops::AddAssign<$name> for $name {
            #[inline]
            fn add_assign(&mut self, other: Self) {
                *self = *self + other;
            }
        }

        impl std::ops::Sub<$name> for $name {
            type Output = Self;
            #[inline]
            fn sub(self, other: Self) -> Self::Output {
                $name::new(self.x - other.x, self.y - other.y, self.z - other.z, self.w - other.w)
            }
        }

        impl std::ops::SubAssign<$name> for $name {
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                *self = *self - other;
            }
        }

        impl std::ops::Mul<$name> for $name {
            type Output = Self;
            #[inline]
            fn mul(self, other: Self) -> Self::Output {
                $name::new(self.x * other.x, self.y * other.y, self.z * other.z, self.w * other.w)
            }
        }

        impl std::ops::MulAssign<$name> for $name {
            #[inline]
            fn mul_assign(&mut self, other: Self) {
                *self = *self * other;
            }
        }

        impl std::ops::Div<$name> for $name {
            type Output = Self;
            #[inline]
            fn div(self, other: Self) -> Self::Output {
                $name::new(self.x / other.x, self.y / other.y, self.z / other.z, self.w / other.w)
            }
        }

        impl std::ops::DivAssign<$name> for $name {
            #[inline]
            fn div_assign(&mut self, other: Self) {
                *self = *self / other;
            }
        }

        #[allow(clippy::modulo_arithmetic)]
        impl std::ops::Rem<$name> for $name {
            type Output = Self;
            #[inline]
            fn rem(self, other: Self) -> Self::Output {
                $name::new(self.x % other.x, self.y % other.y, self.z % other.z, self.w % other.w)
            }
        }

        impl std::ops::RemAssign<$name> for $name {
            #[inline]
            fn rem_assign(&mut self, other: Self) {
                *self = *self % other;
            }
        }
    )*};
}

gen_uivec2!(BVec2 => bool);
numeric_uivec2!(UVec2 => u32, IVec2 => i32);
gen_uivec3!(BVec3A => bool);
numeric_uivec3!(ColorU8RGB => u8, UVec3A => u32, IVec3A => i32);
gen_uivec4!(BVec4 => bool);
numeric_uivec4!(ColorU8RGBA => u8, UVec4 => u32, IVec4 => i32);
