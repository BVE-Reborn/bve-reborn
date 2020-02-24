use crate::parse::kvp::FromKVPValue;
use bve_derive::{FromKVPFile, FromKVPSection, FromKVPValue};

pub mod acceleration;
pub mod delay;
pub mod movement;
pub mod performance;
pub mod version;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedTrainDat {
    #[kvp(bare)]
    pub version: version::VersionSection,
    pub acceleration: acceleration::AccelerationSection,
    #[kvp(alias = "deceleration")]
    pub performance: performance::PerformanceSection,
    pub delay: delay::DelaySection,
    #[kvp(rename = "move")] // move is a keyword
    pub movement: movement::MovementSection,
}
