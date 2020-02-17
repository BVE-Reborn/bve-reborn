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
pub struct KVPFile {
    pub sections: Vec<Section>,
}

impl Default for KVPFile {
    fn default() -> Self {
        Self {
            sections: Vec::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Section {
    pub name: Option<String>,
    pub span: Span,
    pub values: Vec<Value>,
}

impl Default for Section {
    fn default() -> Self {
        Self {
            name: None,
            span: Span::from_line(0),
            values: Vec::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Value {
    pub span: Span,
    pub data: ValueData,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ValueData {
    KeyValuePair { key: String, value: String },
    Value { value: String },
}

pub fn parse_kvp_file(input: &str) -> KVPFile {
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
                    Some(idx) => String::from(line[1..idx].trim()),
                    None => String::from(line[1..].trim()),
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
                        let key = String::from(line[0..idx].trim());
                        let value = String::from(line[(idx + 1)..].trim());
                        ValueData::KeyValuePair { key, value }
                    }
                    // No Equals it's a Value
                    None => {
                        let value = String::from(line.trim());
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
                        data: ValueData::Value {
                            value: String::from("my_value")
                        }
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
                            key: String::from("my_key"),
                            value: String::from("my_value"),
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
                        name: Some(String::from("my_section")),
                        span: Span::from_line(1),
                        values: vec![Value {
                            span: Span::from_line(2),
                            data: ValueData::Value {
                                value: String::from("my_value")
                            }
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
                        name: Some(String::from("my_section")),
                        span: Span::from_line(1),
                        values: vec![Value {
                            span: Span::from_line(2),
                            data: ValueData::KeyValuePair {
                                key: String::from("my_key"),
                                value: String::from("my_value"),
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
                        name: Some(String::from("")),
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
                        name: Some(String::from("my_section")),
                        span: Span::from_line(1),
                        values: Vec::default(),
                    }
                ]
            }
        );
    }
}
