//! Generic parser for key-value pair format. Not quite toml, not quite INI.

use crate::parse::Span;

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

pub struct Value {
    pub span: Span,
    pub data: ValueData,
}

pub enum ValueData {
    KeyValuePair { key: String, value: String },
    Value { value: String },
}

pub fn parse_kvp_file(input: &str) -> KVPFile {
    let input = input.to_lowercase();

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
