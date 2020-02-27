use bve_derive::{FromKVPSection, FromKVPValueEnumNumbers};

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct BrakeSection {
    #[kvp(bare)]
    pub brake_type: BrakeType,
    #[kvp(bare)]
    pub brake_control_system: BrakeControlSystem,
    #[kvp(bare)]
    pub brake_control_speed: f32,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum BrakeType {
    #[kvp(default)]
    /// Electromagnetic straight air brake
    Electromagnetic,
    /// Digital/analog electro-pneumatic air brake without brake pipe (electric command brake)
    ElectricPneumatic,
    /// Air brake with partial release feature
    AirPartial,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum BrakeControlSystem {
    #[kvp(default)]
    None,
    ClosingElectromagneticValve,
    DelayIncludingControl,
}
