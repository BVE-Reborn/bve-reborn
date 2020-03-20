use crate::parse::function_scripts::ParsedFunctionScript;
use crate::parse::kvp::FromKVPValue;
use crate::parse::PrettyPrintResult;
use bve_derive::{FromKVPSection, FromKVPValue};
use cgmath::{Vector2, Vector3};
use num_traits::Zero;
use std::io;

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
        write!(
            out,
            "{}",
            match self {
                TextureOverride::None => "None",
                TextureOverride::Timetable => "Timetable",
            },
        )
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
        write!(
            out,
            "{}",
            match self {
                Self::EveryFrame => "EveryFrame",
                Self::Seconds(v) => return write!(out, "{}s", v),
            }
        )
    }
}
