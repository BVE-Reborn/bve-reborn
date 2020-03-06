use bve_derive::FromKVPFile;
use std::collections::HashMap;

pub mod car;
pub mod coupler;
pub mod exterior;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedExtensionsCfg {
    #[kvp(rename = "car")]
    pub cars: HashMap<u64, car::CarSection>,
    #[kvp(rename = "coupler")]
    pub couplers: HashMap<u64, coupler::CouplerSection>,
    #[kvp(rename = "exterior")]
    pub exteriors: exterior::ExteriorSection,
}
