use bve_derive::FromKVPSection;
use glam::Vec3;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct Includes {
    #[kvp(bare)]
    pub files: Vec<String>,
    pub position: Vec3,
}

impl Default for Includes {
    fn default() -> Self {
        Self {
            files: Vec::default(),
            position: Vec3::zero(),
        }
    }
}
