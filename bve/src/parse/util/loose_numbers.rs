use serde::de::Visitor;
use serde::export::PhantomData;
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct LooseNumber<T>(pub T);

impl<'de, T> Deserialize<'de> for LooseNumber<T>
where
    T: FromStr,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(LooseFloatVisitor { pd: PhantomData::<T> })
    }
}

struct LooseFloatVisitor<T>
where
    T: FromStr,
{
    pd: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for LooseFloatVisitor<T>
where
    T: FromStr,
{
    type Value = LooseNumber<T>;

    fn expecting<'a>(&self, formatter: &mut Formatter<'a>) -> fmt::Result {
        write!(formatter, "Expected loose float.")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut filtered: String = v.chars().filter(|c| !c.is_whitespace()).collect();

        while !filtered.is_empty() {
            let parsed: Result<T, _> = filtered.parse();
            match parsed {
                Ok(v) => return Ok(LooseNumber(v)),
                Err(e) => filtered.pop(),
            };
        }
        Err(serde::de::Error::custom(format!("Error parsing the loose float {}", v)))
    }
}
