use bve_derive::FromKVPSection;
use glam::Vec2;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct PilotLampSection {
    #[kvp(rename = "TurnOn", alias = "点灯")]
    pub on: String,
    #[kvp(rename = "TurnOff", alias = "消灯")]
    pub off: String,
    #[kvp(alias = "左上")]
    pub corner: Vec2,
}

impl Default for PilotLampSection {
    fn default() -> Self {
        Self {
            on: String::default(),
            off: String::default(),
            corner: Vec2::zero(),
        }
    }
}
