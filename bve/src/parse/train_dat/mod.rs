use crate::parse::kvp::{KVPSymbols, DAT_LIKE};
use crate::parse::KVPFileParser;
pub use sections::*;
use std::fmt;

mod sections;

impl fmt::Display for ParsedTrainDat {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

impl KVPFileParser for ParsedTrainDat {
    const SYMBOLS: KVPSymbols = DAT_LIKE;
    const COMMENT: char = ';';
}
