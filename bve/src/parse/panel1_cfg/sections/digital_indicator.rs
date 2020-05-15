use bve_derive::{FromKVPSection, FromKVPValueEnumNumbers};
use glam::Vec2;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct DigitalIndicatorSection {
    pub number: String,
    pub corner: Vec2,
    pub size: Vec2,
    pub unit: Unit,
}

impl Default for DigitalIndicatorSection {
    fn default() -> Self {
        Self {
            number: String::default(),
            corner: Vec2::zero(),
            size: Vec2::zero(),
            unit: Unit::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum Unit {
    #[kvp(default, alias = "km/h")]
    KilometersPerHour,
    #[kvp(alias = "mph")]
    MilesPerHour,
    #[kvp(alias = "m/s")]
    MetersPerSecond,
}
