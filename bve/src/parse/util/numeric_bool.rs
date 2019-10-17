use serde::de;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct NumericBool(pub bool);

impl<'de> Deserialize<'de> for NumericBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i64(NumericBoolVisitor)
    }
}

struct NumericBoolVisitor;

impl<'de> Visitor<'de> for NumericBoolVisitor {
    type Value = NumericBool;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(formatter, "Expecting integer to convert to bool.")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(NumericBool(v != 0))
    }
}
