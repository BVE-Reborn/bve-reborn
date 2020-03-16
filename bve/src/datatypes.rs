use cgmath::{Vector1, Vector2, Vector3, Vector4};
use std::fmt;
use std::str::FromStr;

/// Reinterprets a vector as an integer.
///
/// Used for sorting vectors where the actual order doesn't matter, but _an_ order needs to be made.
pub(crate) trait Asu32 {
    #[must_use]
    fn as_u32(self) -> u32;
}

/// R color: Unsigned 8-bit integer per channel
pub type ColorU8R = Vector1<u8>;
/// RG color: Unsigned 8-bit integer per channel
pub type ColorU8RG = Vector2<u8>;
/// RGB color: Unsigned 8-bit integer per channel
pub type ColorU8RGB = Vector3<u8>;
/// RGBA color: Unsigned 8-bit integer per channel
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

/// R color: Unsigned 16-bit integer per channel
pub type ColorU16R = Vector1<u16>;
/// RG color: Unsigned 16-bit integer per channel
pub type ColorU16RG = Vector2<u16>;
/// RGB color: Unsigned 16-bit integer per channel
pub type ColorU16RGB = Vector3<u16>;
/// RGBA color: Unsigned 16-bit integer per channel
pub type ColorU16RGBA = Vector4<u16>;

/// R color: 32-bit float per channel
pub type ColorF32R = Vector1<f32>;
/// RG color: 32-bit float per channel
pub type ColorF32RG = Vector2<f32>;
/// RGB color: 32-bit float per channel
pub type ColorF32RGB = Vector3<f32>;
/// RGBA color: 32-bit float per channel
pub type ColorF32RGBA = Vector4<f32>;

/// A color that is specifically formatted using normal HTML hex notation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HexColor3(pub ColorU8RGB);

impl HexColor3 {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self(ColorU8RGB::new(r, g, b))
    }
}

impl fmt::Display for HexColor3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:0>6x}", self.0.as_u32())
    }
}

impl FromStr for HexColor3 {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let no_hash = &s[1..];
        let number = u32::from_str_radix(no_hash, 16).map_err(drop)?;

        Ok(Self(ColorU8RGB::new(
            (number & 0xFF) as u8,
            ((number >> 8) & 0xFF) as u8,
            ((number >> 16) & 0xFF) as u8,
        )))
    }
}

/// A color with alpha that is specifically formatted using normal HTML hex notation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HexColor4(pub ColorU8RGBA);

impl HexColor4 {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(ColorU8RGBA::new(r, g, b, a))
    }
}

impl fmt::Display for HexColor4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:0>8x}", self.0.as_u32())
    }
}

impl FromStr for HexColor4 {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let no_hash = &s[1..];
        let number = u32::from_str_radix(no_hash, 16).map_err(drop)?;

        Ok(Self(ColorU8RGBA::new(
            (number & 0xFF) as u8,
            ((number >> 8) & 0xFF) as u8,
            ((number >> 16) & 0xFF) as u8,
            ((number >> 32) & 0xFF) as u8,
        )))
    }
}
