use crate::parse::kvp::{parse_kvp_file, FromKVPFile, KVPGenericWarning, DAT_LIKE};
use crate::parse::util::strip_comments;
pub use sections::*;

mod sections;

#[must_use]
pub fn parse_train_dat(input: &str) -> (ParsedTrainDat, Vec<KVPGenericWarning>) {
    let lower = strip_comments(input, ';').to_lowercase();
    let kvp_file = parse_kvp_file(&lower, DAT_LIKE);

    ParsedTrainDat::from_kvp_file(&kvp_file)
}
