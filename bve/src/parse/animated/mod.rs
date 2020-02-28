use crate::parse::kvp::{parse_kvp_file, FromKVPFile, KVPGenericWarning, ANIMATED_LIKE};
use crate::parse::util::strip_comments;
pub use structs::*;

mod structs;

#[must_use]
pub fn parse_animated_file(input: &str) -> (ParsedAnimatedObject, Vec<KVPGenericWarning>) {
    let lower = strip_comments(input, ';').to_lowercase();
    let kvp_file = parse_kvp_file(&lower, ANIMATED_LIKE);

    ParsedAnimatedObject::from_kvp_file(&kvp_file)
}
