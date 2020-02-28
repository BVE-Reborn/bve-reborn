use bve_derive::{FromKVPSection, FromKVPValueEnumNumbers};

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct HandleSection {
    #[kvp(bare)]
    pub handle_type: HandleType,
    #[kvp(bare)]
    pub power_notches: u64,
    #[kvp(bare)]
    pub brake_notches: u64,
    #[kvp(bare)]
    pub power_notch_reduce_steps: u64,
    #[kvp(bare)]
    pub emergency_brake_handle_behavior: EmergencyBrakeHandleBehavior,
    #[kvp(bare)]
    pub loco_brake_notches: u64,
    #[kvp(bare)]
    pub loco_brake_type: LocoBrakeType,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum HandleType {
    #[kvp(default)]
    Separate,
    Combined,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum EmergencyBrakeHandleBehavior {
    #[kvp(default)]
    NoAction,
    PowerToNeutral,
    ReverserToNeutral,
    PowerAndReverserToNeutral,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum LocoBrakeType {
    #[kvp(default)]
    Combined,
    Independent,
    Blocking,
}
