//! Generic parser for key-value pair format. Not quite toml, not quite INI.
//!
//! No frills format. Does not deal with casing, comments, etc. That must be
//! dealt with ahead of time. This just deserializes the file as is. However
//! whitespace is trimmed off the edges of values
//!
//! There may be arbitrary duplicates.
//!
//! The first section before a section header is always the unnamed section `None`.
//! This differs from an empty section name `Some("")`
//!
//! ```ini
//! value1
//! key1 = some_value
//!
//! [section1]
//! value
//! key = value
//! key = value
//! ```

use crate::parse::Span;
pub use parse::*;

mod parse;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KVPFile<'s> {
    pub sections: Vec<KVPSection<'s>>,
}

impl<'s> Default for KVPFile<'s> {
    fn default() -> Self {
        Self {
            sections: Vec::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KVPSection<'s> {
    pub name: Option<&'s str>,
    pub span: Span,
    pub values: Vec<KVPValue<'s>>,
}

impl<'s> Default for KVPSection<'s> {
    fn default() -> Self {
        Self {
            name: None,
            span: Span::from_line(0),
            values: Vec::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KVPValue<'s> {
    pub span: Span,
    pub data: ValueData<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueData<'s> {
    KeyValuePair { key: &'s str, value: &'s str },
    Value { value: &'s str },
}

pub trait FromKVPFile {
    fn from_kvp_file(k: &KVPFile<'_>) -> Self;
}

pub trait FromKVPSection {
    fn from_kvp_section(section: &KVPSection<'_>) -> Self;
}

pub trait FromKVPData {
    fn from_kvp_data(data: &KVPValue<'_>) -> Self;
}
