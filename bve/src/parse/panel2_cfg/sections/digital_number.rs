use crate::parse::panel2_cfg::{Subject, SubjectTarget};
use crate::HexColor3;
use bve_derive::FromKVPSection;
use cgmath::{Array, Vector2};

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct DigitalNumberSection {
    pub subject: Subject,
    pub location: Vector2<f32>,
    pub daytime_image: String,
    pub nighttime_image: String,
    pub transparent_color: HexColor3,
    pub interval: f32,
    pub layer: i64,
}

impl Default for DigitalNumberSection {
    fn default() -> Self {
        Self {
            subject: Subject::from_target(SubjectTarget::True),
            location: Vector2::from_value(0.0),
            daytime_image: String::new(),
            nighttime_image: String::new(),
            transparent_color: HexColor3::new(0, 0, 255),
            interval: 0.0,
            layer: 0,
        }
    }
}
