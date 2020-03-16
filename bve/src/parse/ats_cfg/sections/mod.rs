use bve_derive::FromKVPFile;

mod plugin;

#[derive(Debug, Default, Clone, PartialEq, FromKVPFile)]
pub struct ParsedAtsConfig {
    #[kvp(bare)]
    pub plugin: plugin::PluginSection,
}
