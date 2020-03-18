use crate::parse::kvp::{parse_kvp_file, FromKVPFile, KVPGenericWarning, ANIMATED_LIKE};
use crate::parse::util::strip_comments;
pub use sections::*;

mod sections;

#[must_use]
pub fn parse_sound_cfg(input: &str) -> (ParsedSoundCfg, Vec<KVPGenericWarning>) {
    let lower = strip_comments(input, ';').to_lowercase();
    let kvp_file = parse_kvp_file(&lower, ANIMATED_LIKE);

    ParsedSoundCfg::from_kvp_file(&kvp_file)
}
