use bve_derive::FromKVPFile;

mod includes;
mod object;
mod sound;
mod state_change_sound;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedAnimatedObject {
    #[kvp(rename = "include")]
    pub includes: Vec<includes::Includes>,
    #[kvp(rename = "object")]
    pub objects: Vec<object::AnimatedObject>,
    #[kvp(rename = "sound")]
    pub sounds: Vec<sound::AnimatedSound>,
    #[kvp(rename = "statechangesound")]
    pub change_state_sounds: Vec<state_change_sound::AnimatedStateChangeSound>,
}
