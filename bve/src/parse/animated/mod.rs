use crate::parse::kvp::{parse_kvp_file, FromKVPFile, KVPGenericWarning};
pub use structs::*;

mod structs;

#[must_use]
pub fn parse_animated_file(input: &str) -> (ParsedAnimatedObject, Vec<KVPGenericWarning>) {
    let kvp_file = parse_kvp_file(input);

    ParsedAnimatedObject::from_kvp_file(&kvp_file)
}
