use crate::parse::panel2_cfg::{Subject, SubjectTarget};
use bve_derive::FromKVPSection;
use cgmath::{Array, Vector2};

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct LinearGaugeSection {
    pub subject: Subject,
    pub location: Vector2<f32>,
    pub minimum: f32,
    pub maximum: f32,
    pub direction: Vector2<i32>,
    pub width: i64,
    pub layer: i64,
}

impl Default for LinearGaugeSection {
    fn default() -> Self {
        Self {
            subject: Subject::from_target(SubjectTarget::True),
            location: Vector2::from_value(0.0),
            minimum: 0.0,
            maximum: 1000.0,
            direction: Vector2::from_value(0),
            width: 0,
            layer: 0,
        }
    }
}
