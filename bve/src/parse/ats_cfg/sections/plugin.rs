use bve_derive::FromKVPSection;

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct PluginSection {
    #[kvp(bare)]
    pub plugin: Option<String>,
}
