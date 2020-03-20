use bve_derive::FromKVPSection;
use cgmath::{Vector3, Zero};

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
