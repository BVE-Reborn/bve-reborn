use bve_derive::FromKVPSection;

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct VersionSection {
    #[kvp(bare)]
    pub version: String,
}
