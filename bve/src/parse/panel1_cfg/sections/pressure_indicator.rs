use crate::parse::kvp::FromKVPValue;
use crate::parse::panel1_cfg::sections::IndicatorType;
use bve_derive::{FromKVPSection, FromKVPValueEnumNumbers};
use cgmath::{Array, Vector2};

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct PressureIndicatorSection {
    #[kvp(rename = "type", alias = "形態")]
    pub indicator_type: IndicatorType,
    #[kvp(alias = "LowerHand; 下針")]
    pub lower_needle: Needle,
    #[kvp(alias = "UpperHand; 上針")]
    pub upper_needle: Needle,
    #[kvp(alias = "中心")]
    pub center: Vector2<f32>,
    #[kvp(alias = "半径")]
    pub radius: f32,
    #[kvp(alias = "背景")]
    pub background: String,
    #[kvp(alias = "ふた")]
    pub cover: String,
    #[kvp(alias = "単位")]
    pub unit: Unit,
    #[kvp(alias = "最小")]
    pub minimum: f32,
    #[kvp(alias = "最大")]
    pub maximum: f32,
    #[kvp(alias = "角度")]
    pub angle: f32,
}

impl Default for PressureIndicatorSection {
    fn default() -> Self {
        Self {
            indicator_type: IndicatorType::default(),
            lower_needle: Needle::default(),
            upper_needle: Needle::default(),
            center: Vector2::from_value(0.0),
            radius: 16.0,
            background: String::default(),
            cover: String::default(),
            unit: Unit::default(),
            minimum: 0.0,
            maximum: 1000.0,
            angle: 45.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Needle {
    pub subject: Subject,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl FromKVPValue for Needle {
    fn from_kvp_value(value: &str) -> Option<Self> {
        let mut iterator = value.split(',').flat_map(|v| v.split(':')).map(str::trim);
        Some(Self {
            subject: Subject::from_kvp_value(iterator.next()?)?,
            red: u8::from_kvp_value(iterator.next()?)?,
            green: u8::from_kvp_value(iterator.next()?)?,
            blue: u8::from_kvp_value(iterator.next()?)?,
        })
    }
}

impl Default for Needle {
    fn default() -> Self {
        Self {
            subject: Subject::default(),
            red: 255,
            green: 255,
            blue: 255,
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum Subject {
    #[kvp(default)]
    NoShow,
    #[kvp(alias = "BC; ブレーキシリンダ")]
    BrakeCylinder,
    #[kvp(alias = "BP; ブレーキ管; 制動管")]
    BrakePipe,
    #[kvp(alias = "ER; 釣り合い空気溜め; 釣り合い空気ダメ; つりあい空気溜め; ツリアイ空気")]
    EqualizingReservoir,
    #[kvp(alias = "MR; 元空気溜め; 元空気ダメ")]
    MainReservoir,
    #[kvp(alias = "SAP; 直通管")]
    StraightAirPipe,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum Unit {
    /// Kilo-pascal
    #[kvp(default, alias = "kpa")]
    Kpa,
    /// Kilogram-force per centimeter squared (98066.5 Pa)
    #[kvp(alias = "kgf/cm2; kgf/cm^2; kg/cm2; kg/cm^2")]
    BrakeCylinder,
}
