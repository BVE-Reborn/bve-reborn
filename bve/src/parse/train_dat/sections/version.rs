use crate::parse::kvp::FromKVPValue;
use bve_derive::FromKVPSection;

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct VersionSection {
    #[kvp(bare)]
    pub version: Version,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Version {
    BVE120,
    BVE121,
    BVE122,
    BVE2,
    OpenBVE { version: String },
}

impl Default for Version {
    fn default() -> Self {
        Self::BVE2
    }
}

impl FromKVPValue for Version {
    fn from_kvp_value(value: &str) -> Option<Self> {
        match value {
            "bve1200000" => Some(Self::BVE120),
            "bve1210000" => Some(Self::BVE121),
            "bve1220000" => Some(Self::BVE122),
            "bve2000000" | "openbve" => Some(Self::BVE2),
            _ => {
                if value.starts_with("openbve") {
                    Some(Self::OpenBVE {
                        version: String::from(&value[7..]),
                    })
                } else {
                    None
                }
            }
        }
    }
}
