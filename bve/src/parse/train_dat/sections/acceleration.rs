use bve_derive::{FromKVPSection, FromKVPValue};

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct AccelerationSection {
    #[kvp(bare)]
    pub acceleration_points: Vec<AccelerationPoint>,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPValue)]
pub struct AccelerationPoint {
    /// A positive floating-point number representing the acceleration at a speed of 0 km/h expressed in km/h/s.
    pub a0: f32,
    /// A positive floating-point number representing the acceleration at a speed of v1 expressed in km/h/s.
    pub a1: f32,
    /// A positive floating-point number representing a reference speed in km/h corresponding to a1.
    pub v1: f32,
    /// A positive floating-point number representing a reference speed in km/h corresponding to e.
    pub v2: f32,
    /// A positive floating-point number representing an exponent. The behavior is different for version 1.22 and
    /// version 2.0 file formats.
    pub e: f32,
}
