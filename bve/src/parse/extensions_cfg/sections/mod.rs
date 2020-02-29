use bve_derive::FromKVPFile;
use std::collections::HashMap;

pub mod car;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedExtensionsCfg {
    #[kvp(rename = "car")]
    pub cars: HashMap<u64, car::CarSection>,
}
