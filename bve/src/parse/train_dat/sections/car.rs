use crate::parse::kvp::FromKVPValue;
use crate::parse::PrettyPrintResult;
use bve_derive::{FromKVPSection, FromKVPValueEnumNumbers};
use std::io;

#[derive(Debug, Clone, PartialEq, FromKVPSection)]
pub struct CarSection {
    #[kvp(bare)]
    motor_car_mass: f32,
    #[kvp(bare)]
    number_of_motor_cars: u64,
    #[kvp(bare)]
    trailer_car_mass: f32,
    #[kvp(bare)]
    number_of_trailer_cars: u64,
    #[kvp(bare)]
    length_of_car: f32,
    #[kvp(bare)]
    front_car_is_motor_car: FrontCarType,
    #[kvp(bare)]
    width_of_car: f32,
    #[kvp(bare)]
    height_of_car: f32,
    #[kvp(bare)]
    center_of_mass_height: f32,
    #[kvp(bare)]
    exposed_frontal_area: FrontalArea,
    #[kvp(bare)]
    unexposed_frontal_area: FrontalArea,
}

impl Default for CarSection {
    fn default() -> Self {
        Self {
            motor_car_mass: 0.0,
            number_of_motor_cars: 0,
            trailer_car_mass: 0.0,
            number_of_trailer_cars: 0,
            length_of_car: 0.0,
            front_car_is_motor_car: FrontCarType::default(),
            width_of_car: 2.6,
            height_of_car: 3.6,
            center_of_mass_height: 1.6,
            exposed_frontal_area: FrontalArea::Calculated,
            unexposed_frontal_area: FrontalArea::Calculated,
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum FrontCarType {
    #[kvp(default)]
    TrailerCar,
    MotorCar,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrontalArea {
    /// exposed: `0.6 * height_of_car * width_of_car`
    /// unexposed: `0.2 * height_of_car * width_of_car`
    Calculated,
    Constant(f32),
}

impl FromKVPValue for FrontalArea {
    fn from_kvp_value(value: &str) -> Option<Self> {
        let value = f32::from_kvp_value(value);
        Some(match value {
            Some(v) if v >= 0.0 => Self::Constant(v),
            _ => Self::Calculated,
        })
    }
}

impl PrettyPrintResult for FrontalArea {
    fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        match self {
            FrontalArea::Calculated => write!(out, "Calculated"),
            FrontalArea::Constant(v) => write!(out, "Constant: {}", v),
        }
    }
}
