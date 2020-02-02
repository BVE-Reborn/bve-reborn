use serde::de;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct LooseNumericBool(pub bool);

impl<'de> Deserialize<'de> for LooseNumericBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(LooseNumericBoolVisitor)
    }
}

struct LooseNumericBoolVisitor;

impl<'de> Visitor<'de> for LooseNumericBoolVisitor {
    type Value = LooseNumericBool;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(formatter, "Expecting integer to convert to bool.")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let mut filtered: String = v.chars().filter(|c| !c.is_whitespace()).collect();

        while !filtered.is_empty() {
            let parsed: Result<i64, _> = filtered.parse();
            match parsed {
                Ok(v) => return Ok(LooseNumericBool(v != 0)),
                Err(_) => filtered.pop(),
            };
        }

        Err(serde::de::Error::custom(format!(
            "Error parsing the numeric bool {}",
            v
        )))
    }
}

#[cfg(test)]
mod test {
    use crate::parse::util::LooseNumericBool;
    use serde_test::{assert_de_tokens, Token};

    #[bve_derive::bve_test]
    #[test]
    fn loose_bool() {
        let b = LooseNumericBool(false);
        assert_de_tokens(&b, &[Token::Str("0")]);
        assert_de_tokens(&b, &[Token::Str("0xxxx0")]);
        assert_de_tokens(&b, &[Token::Str("0.1")]);
        let b = LooseNumericBool(true);
        assert_de_tokens(&b, &[Token::Str("1")]);
        assert_de_tokens(&b, &[Token::Str("1xxxx0")]);
        assert_de_tokens(&b, &[Token::Str("1.1")]);
    }
}
