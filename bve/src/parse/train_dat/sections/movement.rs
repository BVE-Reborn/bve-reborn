use bve_derive::FromKVPSection;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct MovementSection {
    #[kvp(bare)]
    pub jerk_power_up: f32,
    #[kvp(bare)]
    pub jerk_power_down: f32,
    #[kvp(bare)]
    pub jerk_brake_up: f32,
    #[kvp(bare)]
    pub jerk_brake_down: f32,
    #[kvp(bare)]
    pub brake_cylinder_up: f32,
    #[kvp(bare)]
    pub brake_cylinder_down: f32,
}

impl Default for MovementSection {
    fn default() -> Self {
        Self {
            jerk_power_up: 1000.0,
            jerk_power_down: 1000.0,
            jerk_brake_up: 1000.0,
            jerk_brake_down: 1000.0,
            brake_cylinder_up: 300.0,
            brake_cylinder_down: 200.0,
        }
    }
}
