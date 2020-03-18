use crate::HexColorRGB;
use bve_derive::FromKVPSection;
use cgmath::{Array, Vector2};

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct TimetableSection {
    pub location: Vector2<f32>,
    pub width: f32,
    pub height: f32,
    #[kvp(alias = "Transparent")]
    pub transparent_color: HexColorRGB,
    pub layer: i64,
}

impl Default for TimetableSection {
    fn default() -> Self {
        Self {
            location: Vector2::from_value(0.0),
            width: 0.0,
            height: 0.0,
            transparent_color: HexColorRGB::new(0x00, 0x00, 0xFF),
            layer: 0,
        }
    }
}
