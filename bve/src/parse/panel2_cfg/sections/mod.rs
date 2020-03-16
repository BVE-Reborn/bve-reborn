use bve_derive::FromKVPFile;
pub use subject::*;

mod subject;
pub mod this;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedPanel2Cfg {
    pub this: this::ThisSection,
}
