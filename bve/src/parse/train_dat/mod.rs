use crate::parse::kvp::{KVPSymbols, DAT_LIKE};
use crate::parse::KVPFileParser;
pub use sections::*;

mod sections;

impl KVPFileParser for ParsedTrainDat {
    const SYMBOLS: KVPSymbols = DAT_LIKE;
    const COMMENT: char = ';';
}
