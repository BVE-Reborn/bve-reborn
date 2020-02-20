use crate::parse::kvp::{parse_kvp_file, FromKVPFile};
pub use structs::*;

mod structs;

#[must_use]
pub fn parse_animated_file(input: &str) -> ParsedAnimatedObject {
    let kvp_file = parse_kvp_file(input);

    ParsedAnimatedObject::from_kvp_file(&kvp_file)
}
