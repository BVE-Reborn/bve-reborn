use crate::{
    parse::panel2_cfg::{Subject, SubjectTarget},
    IVec2,
};
use bve_derive::FromKVPSection;
use glam::Vec2;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct LinearGaugeSection {
    pub subject: Subject,
    pub location: Vec2,
    pub minimum: f32,
    pub maximum: f32,
    pub direction: IVec2,
    pub width: i64,
    pub layer: i64,
}

impl Default for LinearGaugeSection {
    fn default() -> Self {
        Self {
            subject: Subject::from_target(SubjectTarget::True),
            location: Vec2::zero(),
            minimum: 0.0,
            maximum: 1000.0,
            direction: IVec2::splat(0),
            width: 0,
            layer: 0,
        }
    }
}
