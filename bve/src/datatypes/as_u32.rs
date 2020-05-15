use crate::datatypes::*;

/// Reinterprets a vector as an integer.
///
/// Used for sorting vectors where the actual order doesn't matter, but _an_ order needs to be made.
pub trait Asu32 {
    #[must_use]
    fn as_u32(self) -> u32;
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
