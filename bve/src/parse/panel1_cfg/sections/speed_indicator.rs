use crate::parse::panel1_cfg::sections::IndicatorType;
use bve_derive::{FromKVPSection, FromKVPValue};
use cgmath::{Array, Vector2};

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct SpeedIndicatorSection {
    #[kvp(rename = "type", alias = "形態")]
    pub indicator_type: IndicatorType,
    #[kvp(alias = "中心")]
    pub center: Vector2<f32>,
    #[kvp(alias = "半径")]
    pub radius: f32,
    #[kvp(alias = "背景")]
    pub background: String,
    #[kvp(alias = "ふた")]
    pub cover: String,
    #[kvp(alias = "最小")]
    pub minimum: f32,
    #[kvp(alias = "最大")]
    pub maximum: f32,
    #[kvp(alias = "角度")]
    pub angle: f32,
    #[kvp(alias = "角度")]
    pub atc: String,
    #[kvp(alias = "Atc半径")]
    pub atc_radius: f32,
}

impl Default for SpeedIndicatorSection {
    fn default() -> Self {
        Self {
            indicator_type: IndicatorType::default(),
            center: Vector2::from_value(0.0),
            radius: 16.0,
            background: String::default(),
            cover: String::default(),
            minimum: 0.0,
            maximum: 1000.0,
            angle: 45.0,
            atc: String::default(),
            atc_radius: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromKVPValue)]
pub struct Needle {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Default for Needle {
    fn default() -> Self {
        Self {
            red: 255,
            green: 255,
            blue: 255,
        }
    }
}
