use crate::HexColorRGB;
use bve_derive::FromKVPSection;
use glam::Vec2;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct TimetableSection {
    pub location: Vec2,
    pub width: f32,
    pub height: f32,
    #[kvp(alias = "Transparent")]
    pub transparent_color: HexColorRGB,
    pub layer: i64,
}

impl Default for TimetableSection {
    fn default() -> Self {
        Self {
            location: Vec2::zero(),
            width: 0.0,
            height: 0.0,
            transparent_color: HexColorRGB::new(0x00, 0x00, 0xFF),
            layer: 0,
        }
    }
}
