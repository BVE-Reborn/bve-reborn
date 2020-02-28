use bve_derive::{FromKVPSection, FromKVPValueEnumNumbers};

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct DeviceSection {
    #[kvp(bare)]
    ats: AtsAvailable,
    #[kvp(bare)]
    atc: AtcAvailable,
    #[kvp(bare)]
    eb: EmergencyBrakeAvailable,
    #[kvp(bare)]
    const_speed: ConstSpeedAvailable,
    #[kvp(bare)]
    hold_brake: HoldBrakeAvailable,
    #[kvp(bare)]
    re_adhesion_device: ReAdhesionDeviceType,
    #[kvp(bare)]
    load_compensating_device: String,
    #[kvp(bare)]
    pass_alarm: PassAlarmType,
    #[kvp(bare)]
    door_open_mode: DoorMode,
    #[kvp(bare)]
    door_close_mode: DoorMode,
}

/// Japanese implementation of digital Automatic Train Stops, supports ATS-SN and ATS-P
#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum AtsAvailable {
    #[kvp(default, index = "-1")]
    Neither,
    AtsSn,
    AtsSnAtsP,
}

/// Japanese implementation of digital Automatic Train Control
#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum AtcAvailable {
    #[kvp(default)]
    Unavailable,
    Manual,
    Automatic,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum EmergencyBrakeAvailable {
    #[kvp(default)]
    Unavailable,
    Available,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum ConstSpeedAvailable {
    #[kvp(default)]
    Unavailable,
    Available,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum HoldBrakeAvailable {
    #[kvp(default)]
    Unavailable,
    Available,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum ReAdhesionDeviceType {
    #[kvp(default, index = "-1")]
    Unavailable,
    /// Cuts off power instantly and rebuilds it up fast in steps.
    TypeA,
    /// Updates not so often and adapts slowly. Wheel slip can persist longer and power is regained slower. The
    /// behavior is smoother.
    TypeB,
    /// The behavior is somewhere in-between type B and type D.
    TypeC,
    /// Updates fast and adapts fast. Wheel slip only occurs briefly and power is regained fast. The behavior is more
    /// abrupt.
    TypeD,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum PassAlarmType {
    #[kvp(default)]
    Unavailable,
    Single,
    Loop,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum DoorMode {
    #[kvp(default)]
    AutomaticOrManual,
    AutomaticOnly,
    ManualOnly,
}
