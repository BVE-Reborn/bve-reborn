use crate::parse::kvp::{KVPSymbols, ANIMATED_LIKE};
use crate::parse::KVPFileParser;
pub use sections::*;

mod sections;

impl KVPFileParser for ParsedAtsConfig {
    const SYMBOLS: KVPSymbols = ANIMATED_LIKE;
    const COMMENT: char = ';';
}
