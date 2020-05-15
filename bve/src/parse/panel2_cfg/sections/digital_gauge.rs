use crate::{
    parse::panel2_cfg::{Subject, SubjectTarget},
    HexColorRGB,
};
use bve_derive::FromKVPSection;
use glam::Vec2;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct DigitalGaugeSection {
    pub subject: Subject,
    pub location: Vec2,
    pub radius: f32,
    pub color: HexColorRGB,
    pub initial_angle: f32,
    pub last_angle: f32,
    pub minimum: f32,
    pub maximum: f32,
    pub step: f32,
    pub layer: i64,
}

impl Default for DigitalGaugeSection {
    fn default() -> Self {
        Self {
            subject: Subject::from_target(SubjectTarget::True),
            location: Vec2::zero(),
            radius: 0.0,
            color: HexColorRGB::new(0xFF, 0xFF, 0xFF),
            initial_angle: -120.0,
            last_angle: 120.0,
            minimum: 0.0,
            maximum: 0.0,
            step: 0.0,
            layer: 0,
        }
    }
}
