use bve_derive::FromKVPSection;
use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq, FromKVPSection)]
pub struct ExteriorSection {
    cars: HashMap<u64, String>,
}
