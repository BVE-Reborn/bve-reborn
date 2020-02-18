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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KVPFile<'s> {
    pub sections: Vec<Section<'s>>,
}

impl<'s> Default for KVPFile<'s> {
    fn default() -> Self {
        Self {
            sections: Vec::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Section<'s> {
    pub name: Option<&'s str>,
    pub span: Span,
    pub values: Vec<Value<'s>>,
}

impl<'s> Default for Section<'s> {
    fn default() -> Self {
        Self {
            name: None,
            span: Span::from_line(0),
            values: Vec::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Value<'s> {
    pub span: Span,
    pub data: ValueData<'s>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueData<'s> {
    KeyValuePair { key: &'s str, value: &'s str },
    Value { value: &'s str },
}

#[must_use]
#[allow(clippy::single_match_else)] // This advises less clear code
pub fn parse_kvp_file(input: &str) -> KVPFile<'_> {
    let mut file = KVPFile::default();
    let mut current_section = Section::default();
    for (line_idx, line) in input.lines().enumerate() {
        // Match on the first character
        match line.chars().next() {
            Some('[') => {
                // This is a section
                let end = line.find(']');
                let name = match end {
                    // Allow there to be a missing ] in the section header
                    Some(idx) => line[1..idx].trim(),
                    None => line[1..].trim(),
                };
                // Simultaneously push the previous section and create this new one
                file.sections.push(std::mem::replace(
                    &mut current_section,
                    Section {
                        name: Some(name),
                        span: Span::from_line(line_idx + 1),
                        values: Vec::default(),
                    },
                ));
            }
            Some(..) => {
                // This is a piece of data
                let equals = line.find('=');
                let data = match equals {
                    // Key Value Pair
                    Some(idx) => {
                        let key = line[0..idx].trim();
                        let value = line[(idx + 1)..].trim();
                        ValueData::KeyValuePair { key, value }
                    }
                    // No Equals it's a Value
                    None => {
                        let value = line.trim();
                        ValueData::Value { value }
                    }
                };
                // Push data onto current section
                current_section.values.push(Value {
                    span: Span::from_line(line_idx + 1),
                    data,
                });
            }
            // Empty line, ignore it
            None => {}
        }
    }

    // Push the last section into the file
    file.sections.push(current_section);

    file
}

#[cfg(test)]
mod test {
    use crate::parse::kvp::{parse_kvp_file, KVPFile, Section, Value, ValueData};
    use crate::parse::Span;

    #[test]
    fn value() {
        let kvp = parse_kvp_file("my_value");
        assert_eq!(
            kvp,
            KVPFile {
                sections: vec![Section {
                    name: None,
                    span: Span::from_line(0),
                    values: vec![Value {
                        span: Span::from_line(1),
                        data: ValueData::Value { value: "my_value" }
                    }]
                }]
            }
        );
    }

    #[test]
    fn kvp() {
        let kvp = parse_kvp_file("my_key = my_value");
        assert_eq!(
            kvp,
            KVPFile {
                sections: vec![Section {
                    name: None,
                    span: Span::from_line(0),
                    values: vec![Value {
                        span: Span::from_line(1),
                        data: ValueData::KeyValuePair {
                            key: "my_key",
                            value: "my_value",
                        }
                    }]
                }]
            }
        );
    }

    #[test]
    fn named_section_value() {
        let kvp = parse_kvp_file("[my_section]\nmy_value");
        assert_eq!(
            kvp,
            KVPFile {
                sections: vec![
                    Section {
                        name: None,
                        span: Span::from_line(0),
                        values: Vec::default(),
                    },
                    Section {
                        name: Some("my_section"),
                        span: Span::from_line(1),
                        values: vec![Value {
                            span: Span::from_line(2),
                            data: ValueData::Value { value: "my_value" }
                        }]
                    }
                ]
            }
        );
    }

    #[test]
    fn named_section_kvp() {
        let kvp = parse_kvp_file("[my_section]\nmy_key = my_value");
        assert_eq!(
            kvp,
            KVPFile {
                sections: vec![
                    Section {
                        name: None,
                        span: Span::from_line(0),
                        values: Vec::default(),
                    },
                    Section {
                        name: Some("my_section"),
                        span: Span::from_line(1),
                        values: vec![Value {
                            span: Span::from_line(2),
                            data: ValueData::KeyValuePair {
                                key: "my_key",
                                value: "my_value",
                            }
                        }]
                    }
                ]
            }
        );
    }

    #[test]
    fn empty_section_name() {
        let kvp = parse_kvp_file("[]");
        assert_eq!(
            kvp,
            KVPFile {
                sections: vec![
                    Section {
                        name: None,
                        span: Span::from_line(0),
                        values: Vec::default(),
                    },
                    Section {
                        name: Some(""),
                        span: Span::from_line(1),
                        values: Vec::default(),
                    }
                ]
            }
        );
    }

    #[test]
    fn section_name_no_rbracket() {
        let kvp = parse_kvp_file("[my_section");
        assert_eq!(
            kvp,
            KVPFile {
                sections: vec![
                    Section {
                        name: None,
                        span: Span::from_line(0),
                        values: Vec::default(),
                    },
                    Section {
                        name: Some("my_section"),
                        span: Span::from_line(1),
                        values: Vec::default(),
                    }
                ]
            }
        );
    }
}
