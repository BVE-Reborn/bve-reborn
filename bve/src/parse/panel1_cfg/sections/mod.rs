use bve_derive::{FromKVPFile, FromKVPValueEnumNumbers};

pub mod brake_indicator;
pub mod digital_indicator;
pub mod panel;
pub mod pilot_lamp;
pub mod pressure_indicator;
pub mod speed_indicator;
pub mod version;
pub mod view;
pub mod watch;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedPanel1Cfg {
    #[kvp(bare)]
    pub version: version::VersionSection,
    pub panel: panel::PanelSection,
    pub view: view::ViewSection,
    #[kvp(rename = "PressureIndicator", alias = "PressureGauge; PressureMeter; 圧力計")]
    pub pressure_indicators: Vec<pressure_indicator::PressureIndicatorSection>,
    #[kvp(rename = "SpeedIndicator", alias = "Speedometer; 速度計")]
    pub speed_indicators: Vec<speed_indicator::SpeedIndicatorSection>,
    #[kvp(rename = "DigitalIndicator")]
    pub digital_indicators: Vec<digital_indicator::DigitalIndicatorSection>,
    #[kvp(rename = "PilotLamp", alias = "知らせ灯")]
    pub pilot_lamps: Vec<pilot_lamp::PilotLampSection>,
    #[kvp(rename = "Watch", alias = "時計")]
    pub watches: Vec<watch::WatchSection>,
    #[kvp(rename = "BrakeIndicator", alias = "時計")]
    pub brake_indicators: Vec<brake_indicator::BrakeIndicatorSection>,
}

#[derive(Debug, Clone, PartialEq, FromKVPValueEnumNumbers)]
pub enum IndicatorType {
    #[kvp(default)]
    Gauge,
    LED,
}
