use crate::{
    parse::panel2_cfg::{Subject, SubjectTarget},
    HexColorRGB,
};
use bve_derive::FromKVPSection;
use cgmath::{Array, Vector2};

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct NeedleSection {
    pub subject: Subject,
    pub location: Vector2<f32>,
    pub radius: Option<f32>,
    pub daytime_image: String,
    pub nighttime_image: String,
    pub color: HexColorRGB,
    #[kvp(alias = "Transparent")]
    pub transparent_color: HexColorRGB,
    pub origin: Option<Vector2<f32>>,
    pub initial_angle: f32,
    pub last_angle: f32,
    pub minimum: f32,
    pub maximum: f32,
    pub natural_freq: Option<f32>,
    pub damping_ratio: Option<f32>,
    pub layer: i64,
}

impl Default for NeedleSection {
    fn default() -> Self {
        Self {
            subject: Subject::from_target(SubjectTarget::True),
            location: Vector2::from_value(0.0),
            radius: None,
            daytime_image: String::default(),
            nighttime_image: String::default(),
            color: HexColorRGB::new(255, 255, 255),
            transparent_color: HexColorRGB::new(0, 0, 255),
            origin: None,
            initial_angle: -120.0,
            last_angle: 120.0,
            minimum: 0.0,
            maximum: 1000.0,
            natural_freq: None,
            damping_ratio: None,
            layer: 0,
        }
    }
}
