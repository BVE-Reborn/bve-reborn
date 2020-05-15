use bve_derive::FromKVPSection;
use glam::Vec2;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct BrakeIndicatorSection {
    pub image: String,
    pub corner: Vec2,
    pub width: f32,
}

impl Default for BrakeIndicatorSection {
    fn default() -> Self {
        Self {
            image: String::default(),
            corner: Vec2::zero(),
            width: 0.0,
        }
    }
}
