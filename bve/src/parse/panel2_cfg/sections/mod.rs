use bve_derive::FromKVPFile;
pub use subject::*;

pub mod needle;
pub mod pilot_lamp;
mod subject;
pub mod this;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedPanel2Cfg {
    pub this: this::ThisSection,
    pub pilot_lamp: pilot_lamp::PilotLampSection,
}
