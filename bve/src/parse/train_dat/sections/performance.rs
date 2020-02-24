use crate::parse::kvp::FromKVPValue;
use bve_derive::{FromKVPFile, FromKVPSection, FromKVPValue};

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct PerformanceSection {
    #[kvp(bare)]
    pub deceleration: f32,
    #[kvp(bare)]
    pub coefficient_of_static_friction: f32,
    #[kvp(bare)]
    pub _reserved0: f32,
    #[kvp(bare)]
    pub coefficient_of_rolling_resistance: f32,
    #[kvp(bare)]
    pub aerodynamic_drag_coefficient: f32,
}

impl Default for PerformanceSection {
    fn default() -> Self {
        Self {
            deceleration: 1.0,
            coefficient_of_static_friction: 0.35,
            _reserved0: 0.0,
            coefficient_of_rolling_resistance: 0.0025,
            aerodynamic_drag_coefficient: 1.1,
        }
    }
}
