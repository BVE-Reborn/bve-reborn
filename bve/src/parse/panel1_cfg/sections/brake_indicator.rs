use bve_derive::FromKVPSection;
use cgmath::{Array, Vector2};

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct BrakeIndicatorSection {
    pub image: String,
    pub corner: Vector2<f32>,
    pub width: f32,
}

impl Default for BrakeIndicatorSection {
    fn default() -> Self {
        Self {
            file: String::default(),
            corner: Vector2::from_value(0.0),
            width: 0.0,
        }
    }
}
