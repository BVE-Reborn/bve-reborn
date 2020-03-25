use crate::parse::kvp::FromKVPValue;
use crate::parse::PrettyPrintResult;
use bve_derive::FromKVPSection;
use cgmath::Vector3;
use num_traits::Zero;
use std::io;

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

impl PrettyPrintResult for PlayOn {
    fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        writeln!(
            out,
            "{}",
            match self {
                Self::Silent => "Silent",
                Self::Play => "Play",
            },
        )
    }
}
