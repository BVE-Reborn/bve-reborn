use bve_derive::{FromKVPSection, FromKVPValue};

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct CarSection {
    #[kvp(rename = "object")]
    pub object_filename: Option<String>,
    pub length: f32,
    #[kvp(rename = "axles")]
    pub axle_positions: AxlePositions,
    pub reversed: bool,
    pub loading_sway: bool,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPValue)]
pub struct AxlePositions {
    pub rear: f32,
    pub front: f32,
}
