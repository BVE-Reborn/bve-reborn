use crate::parse::{function_scripts::ParsedFunctionScript, kvp::FromKVPValue, PrettyPrintResult};
use bve_derive::{FromKVPSection, FromKVPValue};
use glam::{Vec2, Vec3A};
use std::io;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct AnimatedObject {
    pub position: Vec3A,
    #[kvp(variadic)]
    pub states: Vec<String>,
    pub state_function: Option<ParsedFunctionScript>,

    pub translate_x_direction: Vec3A,
    pub translate_y_direction: Vec3A,
    pub translate_z_direction: Vec3A,

    pub translate_x_function: Option<ParsedFunctionScript>,
    pub translate_y_function: Option<ParsedFunctionScript>,
    pub translate_z_function: Option<ParsedFunctionScript>,

    pub rotate_x_direction: Vec3A,
    pub rotate_y_direction: Vec3A,
    pub rotate_z_direction: Vec3A,

    pub rotate_x_function: Option<ParsedFunctionScript>,
    pub rotate_y_function: Option<ParsedFunctionScript>,
    pub rotate_z_function: Option<ParsedFunctionScript>,

    pub rotate_x_damping: Option<Damping>,
    pub rotate_y_damping: Option<Damping>,
    pub rotate_z_damping: Option<Damping>,

    pub texture_shift_x_direction: Vec2,
    pub texture_shift_y_direction: Vec2,

    pub texture_shift_x_function: Option<ParsedFunctionScript>,
    pub texture_shift_y_function: Option<ParsedFunctionScript>,

    pub track_follower_function: Option<ParsedFunctionScript>,

    pub texture_override: TextureOverride,

    pub refresh_rate: RefreshRate,
}

impl Default for AnimatedObject {
    fn default() -> Self {
        Self {
            position: Vec3A::zero(),
            states: Vec::default(),
            state_function: None,
            translate_x_direction: Vec3A::unit_x(),
            translate_y_direction: Vec3A::unit_y(),
            translate_z_direction: Vec3A::unit_z(),
            translate_x_function: None,
            translate_y_function: None,
            translate_z_function: None,
            rotate_x_direction: Vec3A::unit_x(),
            rotate_y_direction: Vec3A::unit_y(),
            rotate_z_direction: Vec3A::unit_z(),
            rotate_x_function: None,
            rotate_y_function: None,
            rotate_z_function: None,
            rotate_x_damping: None,
            rotate_y_damping: None,
            rotate_z_damping: None,
            texture_shift_x_direction: Vec2::unit_x(),
            texture_shift_y_direction: Vec2::unit_y(),
            texture_shift_x_function: None,
            texture_shift_y_function: None,
            track_follower_function: None,
            texture_override: TextureOverride::default(),
            refresh_rate: RefreshRate::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromKVPValue)]
pub struct Damping {
    pub frequency: f32,
    pub damping_ratio: f32,
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

impl PrettyPrintResult for TextureOverride {
    fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        writeln!(out, "{}", match self {
            Self::None => "None",
            Self::Timetable => "Timetable",
        },)
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

impl PrettyPrintResult for RefreshRate {
    fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        match self {
            Self::EveryFrame => writeln!(out, "Every Frame"),
            Self::Seconds(v) => writeln!(out, "{}s", v),
        }
    }
}
