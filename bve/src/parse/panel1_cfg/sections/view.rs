use bve_derive::FromKVPSection;

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct ViewSection {
    pub yaw: f32,
    pub pitch: f32,
}
