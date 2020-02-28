use bve_derive::FromKVPSection;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct PressureSection {
    #[kvp(bare)]
    brake_cylinder_service_maximum_pressure: f32,
    #[kvp(bare)]
    brake_cylinder_emergency_maximum_pressure: f32,
    #[kvp(bare)]
    main_reservoir_minimum_pressure: f32,
    #[kvp(bare)]
    main_reservoir_maximum_pressure: f32,
    #[kvp(bare)]
    brake_pipe_normal_pressure: f32,
}

impl Default for PressureSection {
    fn default() -> Self {
        Self {
            brake_cylinder_service_maximum_pressure: 480.0,
            brake_cylinder_emergency_maximum_pressure: 480.0,
            main_reservoir_minimum_pressure: 690.0,
            main_reservoir_maximum_pressure: 780.0,
            brake_pipe_normal_pressure: 490.0,
        }
    }
}
