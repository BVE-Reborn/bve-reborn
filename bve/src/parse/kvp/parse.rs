use crate::parse::kvp::{KVPFile, KVPSection, KVPValue, ValueData};
use crate::parse::Span;

#[must_use]
#[allow(clippy::single_match_else)] // This advises less clear code
pub fn parse_kvp_file(input: &str) -> KVPFile<'_> {
    let mut file = KVPFile::default();
    let mut current_section = KVPSection::default();
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
                    KVPSection {
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
                current_section.values.push(KVPValue {
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
    use crate::parse::kvp::{parse_kvp_file, KVPFile, KVPSection, KVPValue, ValueData};
    use crate::parse::Span;

    #[test]
    fn value() {
        let kvp = parse_kvp_file("my_value");
        assert_eq!(
            kvp,
            KVPFile {
                sections: vec![KVPSection {
                    name: None,
                    span: Span::from_line(0),
                    values: vec![KVPValue {
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
                sections: vec![KVPSection {
                    name: None,
                    span: Span::from_line(0),
                    values: vec![KVPValue {
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
                    KVPSection {
                        name: None,
                        span: Span::from_line(0),
                        values: Vec::default(),
                    },
                    KVPSection {
                        name: Some("my_section"),
                        span: Span::from_line(1),
                        values: vec![KVPValue {
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
                    KVPSection {
                        name: None,
                        span: Span::from_line(0),
                        values: Vec::default(),
                    },
                    KVPSection {
                        name: Some("my_section"),
                        span: Span::from_line(1),
                        values: vec![KVPValue {
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
                    KVPSection {
                        name: None,
                        span: Span::from_line(0),
                        values: Vec::default(),
                    },
                    KVPSection {
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
                    KVPSection {
                        name: None,
                        span: Span::from_line(0),
                        values: Vec::default(),
                    },
                    KVPSection {
                        name: Some("my_section"),
                        span: Span::from_line(1),
                        values: Vec::default(),
                    }
                ]
            }
        );
    }
}
