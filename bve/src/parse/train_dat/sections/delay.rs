use bve_derive::FromKVPSection;

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct DelaySection {
    #[kvp(bare, variadic)]
    pub delay_power_up: Vec<f32>,
    #[kvp(bare, variadic)]
    pub delay_power_down: Vec<f32>,
    #[kvp(bare, variadic)]
    pub delay_brake_up: Vec<f32>,
    #[kvp(bare, variadic)]
    pub delay_brake_down: Vec<f32>,
}
