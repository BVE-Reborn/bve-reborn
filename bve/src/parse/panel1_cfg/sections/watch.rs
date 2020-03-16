use bve_derive::{FromKVPSection, FromKVPValue};
use cgmath::{Array, Vector2};

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct WatchSection {
    #[kvp(alias = "背景")]
    pub background: String,
    #[kvp(alias = "中心")]
    pub center: Vector2<f32>,
    #[kvp(alias = "半径")]
    pub radius: f32,
    #[kvp(alias = "Hand; 針")]
    pub needle: Needle,
}

impl Default for WatchSection {
    fn default() -> Self {
        Self {
            background: String::default(),
            center: Vector2::from_value(0.0),
            radius: 16.0,
            needle: Needle::default(),
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
