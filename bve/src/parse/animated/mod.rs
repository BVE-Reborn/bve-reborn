use crate::parse::function_scripts::ParsedFunctionScript;
use crate::parse::kvp::FromKVPValue;
use bve_derive::{FromKVPFile, FromKVPSection};
use cgmath::{Vector2, Vector3};
use num_traits::identities::Zero;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedAnimatedObject {
    pub includes: Includes,
    #[kvp(rename = "object")]
    pub objects: Vec<AnimatedObject>,
    #[kvp(rename = "sound")]
    pub sounds: Vec<AnimatedSound>,
}

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct Includes {
    #[kvp(bare)]
    pub files: Vec<String>,
    pub position: Vector3<f32>,
}

impl Default for Includes {
    fn default() -> Self {
        Self {
            files: Vec::default(),
            position: Vector3::zero(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct AnimatedObject {
    pub position: Vector3<f32>,
    #[kvp(variadic)]
    pub states: Vec<String>,
    pub state_function: Option<ParsedFunctionScript>,

    pub translate_x_direction: Vector3<f32>,
    pub translate_y_direction: Vector3<f32>,
    pub translate_z_direction: Vector3<f32>,

    pub translate_x_function: Option<ParsedFunctionScript>,
    pub translate_y_function: Option<ParsedFunctionScript>,
    pub translate_z_function: Option<ParsedFunctionScript>,

    pub rotate_x_direction: Vector3<f32>,
    pub rotate_y_direction: Vector3<f32>,
    pub rotate_z_direction: Vector3<f32>,

    pub rotate_x_function: Option<ParsedFunctionScript>,
    pub rotate_y_function: Option<ParsedFunctionScript>,
    pub rotate_z_function: Option<ParsedFunctionScript>,

    pub rotate_x_damping: Option<Damping>,
    pub rotate_y_damping: Option<Damping>,
    pub rotate_z_damping: Option<Damping>,

    pub texture_shift_x_direction: Vector2<f32>,
    pub texture_shift_y_direction: Vector2<f32>,

    pub texture_shift_x_function: Option<ParsedFunctionScript>,
    pub texture_shift_y_function: Option<ParsedFunctionScript>,

    pub track_follower_function: Option<ParsedFunctionScript>,

    pub texture_override: TextureOverride,

    pub refresh_rate: RefreshRate,
}

impl Default for AnimatedObject {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            states: Vec::default(),
            state_function: None,
            translate_x_direction: Vector3::unit_x(),
            translate_y_direction: Vector3::unit_y(),
            translate_z_direction: Vector3::unit_z(),
            translate_x_function: None,
            translate_y_function: None,
            translate_z_function: None,
            rotate_x_direction: Vector3::unit_x(),
            rotate_y_direction: Vector3::unit_y(),
            rotate_z_direction: Vector3::unit_z(),
            rotate_x_function: None,
            rotate_y_function: None,
            rotate_z_function: None,
            rotate_x_damping: None,
            rotate_y_damping: None,
            rotate_z_damping: None,
            texture_shift_x_direction: Vector2::unit_x(),
            texture_shift_y_direction: Vector2::unit_y(),
            texture_shift_x_function: None,
            texture_shift_y_function: None,
            track_follower_function: None,
            texture_override: TextureOverride::default(),
            refresh_rate: RefreshRate::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct AnimatedSound {
    filename: String,
    position: Vector3<f32>,
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
            position: Vector3::zero(),
            volume: 1.0,
            pitch: 1.0,
            radius: 30.0,
            volume_function: None,
            pitch_function: None,
            track_follower_function: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct AnimatedStateChangeSound {
    filename: String,
    #[kvp(variadic)]
    filenames: Vec<String>,
    position: Vector3<f32>,
    volume: f32,
    pitch: f32,
    radius: f32,
    play_on_show: PlayOn,
    play_on_hide: PlayOn,
}

impl Default for AnimatedStateChangeSound {
    fn default() -> Self {
        Self {
            filename: String::new(),
            filenames: Vec::new(),
            position: Vector3::zero(),
            volume: 1.0,
            pitch: 1.0,
            radius: 30.0,
            play_on_show: PlayOn::Silent,
            play_on_hide: PlayOn::Silent,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Damping {
    pub frequency: f32,
    pub damping_ratio: f32,
}

impl FromKVPValue for Damping {
    fn from_kvp_value(value: &str) -> Option<Self> {
        Vector2::<f32>::from_kvp_value(value).map(|vec| Self {
            frequency: vec.x,
            damping_ratio: vec.y,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextureOverride {
    None,
    Timetable,
}

impl Default for TextureOverride {
    fn default() -> Self {
        Self::None
    }
}

impl FromKVPValue for TextureOverride {
    fn from_kvp_value(value: &str) -> Option<Self> {
        if value == "timetable" {
            Some(Self::Timetable)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RefreshRate {
    EveryFrame,
    Seconds(f32),
}

impl Default for RefreshRate {
    fn default() -> Self {
        Self::EveryFrame
    }
}

impl FromKVPValue for RefreshRate {
    fn from_kvp_value(value: &str) -> Option<Self> {
        f32::from_kvp_value(value).map(|f| if f == 0.0 { Self::EveryFrame } else { Self::Seconds(f) })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayOn {
    Silent,
    Play,
}

impl Default for PlayOn {
    fn default() -> Self {
        Self::Silent
    }
}

impl FromKVPValue for PlayOn {
    fn from_kvp_value(value: &str) -> Option<Self> {
        i64::from_kvp_value(value).and_then(|i| {
            if i == 0 {
                Some(Self::Silent)
            } else if i == 1 {
                Some(Self::Play)
            } else {
                None
            }
        })
    }
}
