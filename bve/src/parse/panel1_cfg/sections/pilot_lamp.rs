use bve_derive::FromKVPSection;
use cgmath::{Array, Vector2};

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct PilotLampSection {
    #[kvp(rename = "TurnOn", alias = "点灯")]
    pub on: String,
    #[kvp(rename = "TurnOff", alias = "消灯")]
    pub off: String,
    #[kvp(alias = "左上")]
    pub corner: Vector2<f32>,
}

impl Default for PilotLampSection {
    fn default() -> Self {
        Self {
            on: String::default(),
            off: String::default(),
            corner: Vector2::from_value(0.0),
        }
    }
}
