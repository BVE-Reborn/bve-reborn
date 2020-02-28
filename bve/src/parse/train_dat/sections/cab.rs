use bve_derive::FromKVPSection;

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct CabSection {
    /// A floating-point number measured in millimeters (mm) which gives the X-coordinate of the driver’s eye from the
    /// center of the driver’s car. Negative values indicate a location on the left side of the train, positive ones on
    /// the right side.
    #[kvp(bare)]
    x: f32,
    /// A floating-point number measured in millimeters (mm) which gives the Y-coordinate of the driver’s eye from the
    /// top of the rails. Negative values indicate a location below the top of the rails, positive ones above the top
    /// of the rails.
    #[kvp(bare)]
    y: f32,
    /// A floating-point number measured in millimeters (mm) which gives the Z-coordinate of the driver’s eye from the
    /// front of the driver’s car. Negative values indicate a location inside the car, positive ones outside.
    #[kvp(bare)]
    z: f32,
    /// A non-negative integer indicating which car the driver is located in. The first car in the train has index 0,
    /// the second car index 1, and so on.
    #[kvp(bare)]
    car: u64,
}
