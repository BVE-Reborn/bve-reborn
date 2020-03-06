use bve_derive::{FromKVPSection, FromKVPValue};

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct CouplerSection {
    pub distances: Distances,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPValue)]
pub struct Distances {
    minimum: f32,
    maximum: f32,
}
