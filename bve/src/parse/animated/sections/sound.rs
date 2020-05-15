use crate::parse::function_scripts::ParsedFunctionScript;
use bve_derive::FromKVPSection;
use glam::Vec3;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct AnimatedSound {
    filename: String,
    position: Vec3,
    volume: f32,
    volume_function: Option<ParsedFunctionScript>,
    pitch: f32,
    pitch_function: Option<ParsedFunctionScript>,
    radius: f32,
    track_follower_function: Option<ParsedFunctionScript>,
}

impl Default for AnimatedSound {
    fn default() -> Self {
        Self {
            filename: String::new(),
            position: Vec3::zero(),
            volume: 1.0,
            pitch: 1.0,
            radius: 30.0,
            volume_function: None,
            pitch_function: None,
            track_follower_function: None,
        }
    }
}
