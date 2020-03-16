use bve_derive::{FromKVPSection, FromKVPValueEnumNumbers};
use cgmath::{Array, Vector2};

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct DigitalIndicatorSection {
    pub number: String,
    pub corner: Vector2<f32>,
    pub size: Vector2<f32>,
    pub unit: Unit,
}

impl Default for DigitalIndicatorSection {
    fn default() -> Self {
        Self {
            number: String::default(),
            corner: Vector2::from_value(0.0),
            size: Vector2::from_value(0.0),
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
