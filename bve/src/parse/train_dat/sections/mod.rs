use bve_derive::FromKVPFile;

pub mod acceleration;
pub mod brake;
pub mod cab;
pub mod car;
pub mod delay;
pub mod device;
pub mod handle;
pub mod motor;
pub mod movement;
pub mod performance;
pub mod pressure;
pub mod version;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedTrainDat {
    #[kvp(bare)]
    pub version: version::VersionSection,
    pub acceleration: acceleration::AccelerationSection,
    #[kvp(alias = "deceleration")]
    pub performance: performance::PerformanceSection,
    pub delay: delay::DelaySection,
    #[kvp(rename = "move")] // move is a keyword
    pub movement: movement::MovementSection,
    pub brake: brake::BrakeSection,
    pub pressure: pressure::PressureSection,
    pub handle: handle::HandleSection,
    #[kvp(alias = "cab; cockpit")]
    pub cab: cab::CabSection,
    pub car: car::CarSection,
    pub device: device::DeviceSection,
    #[kvp(rename = "motor_p1")]
    pub motor_p1: motor::MotorSection,
    #[kvp(rename = "motor_p2")]
    pub motor_p2: motor::MotorSection,
    #[kvp(rename = "motor_b1")]
    pub motor_b1: motor::MotorSection,
    #[kvp(rename = "motor_b2")]
    pub motor_b2: motor::MotorSection,
}
