use crate::parse::{
    kvp::{KVPSymbols, DAT_LIKE},
    KVPFileParser,
};
pub use sections::*;

mod sections;

impl KVPFileParser for ParsedTrainDat {
    const COMMENT: char = ';';
    const SYMBOLS: KVPSymbols = DAT_LIKE;
}
