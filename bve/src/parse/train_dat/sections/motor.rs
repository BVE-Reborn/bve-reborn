use bve_derive::{FromKVPSection, FromKVPValue};

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct MotorSection {
    #[kvp(bare)]
    sound_data: Vec<MotorSoundType>,
}

#[derive(Debug, Clone, PartialEq, FromKVPValue)]
pub struct MotorSoundType {
    sound_index: i64,
    /// Really just sound speed
    pitch: f32,
    volume: f32,
}
