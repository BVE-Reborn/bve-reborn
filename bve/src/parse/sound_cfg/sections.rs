use bve_derive::{FromKVPFile, FromKVPSection};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedSoundCfg {
    #[kvp(bare)]
    pub version: VersionSection,
    pub run: RunSection,
    pub flange: FlangeSection,
    pub motor: MotorSection,
    pub switch: SwitchSection,
    pub brake: BrakeSection,
    pub compressor: CompressorSection,
    pub suspension: SuspensionSection,
    pub horn: HornSection,
    pub door: DoorSection,
    pub ats: AtsSection,
    pub buzzer: BuzzerSection,
    #[kvp(alias = "Pilot Lamp")]
    pub pilot_lamp: PilotLampSection,
    #[kvp(alias = "Brake Handle")]
    pub brake_handle: BrakeHandleSection,
    #[kvp(alias = "Master Controller")]
    pub master_controller: MasterControllerSection,
    pub reverser: ReverserSection,
    pub breaker: BreakerSection,
    pub others: OthersSection,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct VersionSection {
    #[kvp(bare)]
    pub version: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct RunSection {
    pub run_sounds: HashMap<u64, String>,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct FlangeSection {
    pub flange_sounds: HashMap<u64, String>,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct MotorSection {
    pub motor_sounds: HashMap<u64, String>,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct SwitchSection {
    pub switch_sounds: HashMap<u64, String>,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct BrakeSection {
    #[kvp(alias = "BC Release High")]
    pub brake_cylinder_release_high: String,
    #[kvp(alias = "BC Release")]
    pub brake_cylinder_release: String,
    #[kvp(alias = "BC Release Full")]
    pub brake_cylinder_release_full: String,
    pub emergency: String,
    #[kvp(alias = "BP Decomp")]
    pub brake_pipe_decompression: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct CompressorSection {
    pub attack: String,
    #[kvp(alias = "Loop")]
    pub loop_sound: String,
    pub release: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct SuspensionSection {
    pub left: String,
    pub right: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct HornSection {
    pub primary_start: String,
    pub secondary_start: String,
    pub music_start: String,
    pub primary_loop: String,
    pub secondary_loop: String,
    pub music_loop: String,
    pub primary_end: String,
    pub secondary_end: String,
    pub music_end: String,
    pub primary: String,
    pub secondary: String,
    pub music: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct DoorSection {
    #[kvp(alias = "Open Left")]
    pub open_left: String,
    #[kvp(alias = "Open Right")]
    pub open_right: String,
    #[kvp(alias = "Close Left")]
    pub close_left: String,
    #[kvp(alias = "Close Right")]
    pub close_right: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct AtsSection {
    pub ats_sounds: HashMap<u64, String>,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct BuzzerSection {
    pub correct: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct PilotLampSection {
    pub on: String,
    pub off: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct BrakeHandleSection {
    pub apply: String,
    pub apply_fast: String,
    pub release: String,
    pub release_fast: String,
    pub min: String,
    pub max: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct MasterControllerSection {
    pub up: String,
    pub up_fast: String,
    pub down: String,
    pub down_fast: String,
    pub min: String,
    pub max: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct ReverserSection {
    pub on: String,
    pub off: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct BreakerSection {
    pub on: String,
    pub off: String,
}

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct OthersSection {
    pub noise: String,
    pub shoe: String,
    pub halt: String,
}
