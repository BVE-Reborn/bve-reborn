use bve_derive::FromKVPFile;
pub use subject::*;

pub mod digital_gauge;
pub mod digital_number;
pub mod linear_gauge;
pub mod needle;
pub mod pilot_lamp;
mod subject;
pub mod this;
pub mod timetable;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedPanel2Cfg {
    pub this: this::ThisSection,
    #[kvp(rename = "PilotLamp")]
    pub pilot_lamps: Vec<pilot_lamp::PilotLampSection>,
    #[kvp(rename = "Needle")]
    pub needles: Vec<needle::NeedleSection>,
    #[kvp(rename = "DigitalNumber")]
    pub digital_numbers: Vec<digital_number::DigitalNumberSection>,
    #[kvp(rename = "DigitalGauge")]
    pub digital_gauges: Vec<digital_gauge::DigitalGaugeSection>,
    #[kvp(rename = "LinearGauge")]
    pub linear_gauges: Vec<linear_gauge::LinearGaugeSection>,
    pub timetable: Option<timetable::TimetableSection>,
}
