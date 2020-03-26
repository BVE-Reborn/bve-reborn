use crate::parse::{
    kvp::{KVPSymbols, ANIMATED_LIKE},
    KVPFileParser,
};
pub use sections::*;

mod sections;

impl KVPFileParser for ParsedAnimatedObject {
    const COMMENT: char = ';';
    const SYMBOLS: KVPSymbols = ANIMATED_LIKE;
}
