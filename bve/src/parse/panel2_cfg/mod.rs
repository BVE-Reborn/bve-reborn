use crate::parse::{
    kvp::{KVPSymbols, ANIMATED_LIKE},
    KVPFileParser,
};
pub use sections::*;

mod sections;

impl KVPFileParser for ParsedPanel2Cfg {
    const COMMENT: char = ';';
    const SYMBOLS: KVPSymbols = ANIMATED_LIKE;
}
