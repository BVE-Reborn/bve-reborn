use cgmath::{Vector1, Vector2, Vector3, Vector4};

pub trait Asu32 {
    #[must_use]
    fn as_u32(self) -> u32;
}

pub type ColorU8R = Vector1<u8>;
pub type ColorU8RG = Vector2<u8>;
pub type ColorU8RGB = Vector3<u8>;
pub type ColorU8RGBA = Vector4<u8>;

impl Asu32 for ColorU8R {
    #[must_use]
    fn as_u32(self) -> u32 {
        u32::from(self.x)
    }
}

impl Asu32 for ColorU8RG {
    #[must_use]
    fn as_u32(self) -> u32 {
        u32::from(self.y) << 8 | u32::from(self.x)
    }
}

impl Asu32 for ColorU8RGB {
    #[must_use]
    fn as_u32(self) -> u32 {
        u32::from(self.z) << 16 | u32::from(self.y) << 8 | u32::from(self.x)
    }
}

impl Asu32 for ColorU8RGBA {
    #[must_use]
    fn as_u32(self) -> u32 {
        u32::from(self.w) << 24 | u32::from(self.z) << 16 | u32::from(self.y) << 8 | u32::from(self.x)
    }
}

pub type ColorU16R = Vector1<u16>;
pub type ColorU16RG = Vector2<u16>;
pub type ColorU16RGB = Vector3<u16>;
pub type ColorU16RGBA = Vector4<u16>;

pub type ColorF32R = Vector1<f32>;
pub type ColorF32RG = Vector2<f32>;
pub type ColorF32RGB = Vector3<f32>;
pub type ColorF32RGBA = Vector4<f32>;
