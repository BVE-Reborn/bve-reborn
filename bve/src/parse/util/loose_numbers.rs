use num_traits::Zero;
use serde::{de::Visitor, export::PhantomData, Deserialize, Deserializer};
use std::{
    fmt,
    fmt::{Display, Formatter},
    str::FromStr,
};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
#[repr(transparent)]
pub struct LooseNumber<T>(pub T);

impl<'de, T> Deserialize<'de> for LooseNumber<T>
where
    T: FromStr + Zero + Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(LooseNumberVisitor { pd: PhantomData::<T> })
    }
}

struct LooseNumberVisitor<T>
where
    T: FromStr,
{
    pd: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for LooseNumberVisitor<T>
where
    T: FromStr + Zero + Display,
{
    type Value = LooseNumber<T>;

    fn expecting<'a>(&self, formatter: &mut Formatter<'a>) -> fmt::Result {
        write!(formatter, "Expected loose number")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let parsed = parse_loose_number::<T>(v);

        parsed.ok_or_else(|| serde::de::Error::custom(format!("Error parsing the loose number {}", v)))
    }
}

pub fn parse_loose_number<T>(input: &str) -> Option<LooseNumber<T>>
where
    T: FromStr + Zero + Display,
{
    tracing::trace!(input, "Parsing loose number...");

    let mut filtered: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    while !filtered.is_empty() {
        let parsed: Result<T, _> = filtered.parse();
        match parsed {
            Ok(v) => {
                tracing::trace!(output = %v, %filtered, "Parsed loose number");
                return Some(LooseNumber(v));
            }
            Err(_) => {
                // Allow a single dot to represent 0.0
                if filtered == "." {
                    tracing::trace!(output = 0, %filtered, "Parsed loose number");
                    return Some(LooseNumber(T::zero()));
                } else {
                    filtered.pop();
                }
            }
        };
    }

    None
}

#[cfg(test)]
mod test {
    use crate::parse::util::LooseNumber;
    use serde_test::{assert_de_tokens, Token};

    #[bve_derive::bve_test]
    #[test]
    fn loose_number_f32() {
        let l = LooseNumber::<f32>(1.2);
        assert_de_tokens(&l, &[Token::Str("1.2")]);
        assert_de_tokens(&l, &[Token::Str("1.2000000000000")]);
        assert_de_tokens(&l, &[Token::Str("1.2x8")]);
        assert_de_tokens(&l, &[Token::Str("1    .    2")]);
        assert_de_tokens(&l, &[Token::Str("1    .    2 oh yeah!")]);
        let l = LooseNumber::<f32>(1.6E12);
        assert_de_tokens(&l, &[Token::Str("1.6E12")]);
        assert_de_tokens(&l, &[Token::Str("1.6000000000000E12")]);
        assert_de_tokens(&l, &[Token::Str("1.6E12x8")]);
        assert_de_tokens(&l, &[Token::Str("1    .    6         E        12")]);
        assert_de_tokens(&l, &[Token::Str("1    .    6         E        12 oh yeah!")]);
        let l = LooseNumber::<f32>(1.0);
        assert_de_tokens(&l, &[Token::Str("1")]);
        assert_de_tokens(&l, &[Token::Str("1 . ")]);
        assert_de_tokens(&l, &[Token::Str("1 . 0")]);
        assert_de_tokens(&l, &[Token::Str("1 . 0  E  0")]);
        assert_de_tokens(&l, &[Token::Str("1 . 0  E  0 oh yeah!")]);
    }

    #[bve_derive::bve_test]
    #[test]
    fn loose_number_f32_dot() {
        let l = LooseNumber::<f32>(0.0);
        assert_de_tokens(&l, &[Token::Str(".")]);
        assert_de_tokens(&l, &[Token::Str("    .     ")]);
        assert_de_tokens(&l, &[Token::Str("    .      oh yeah")]);
    }

    #[bve_derive::bve_test]
    #[test]
    fn loose_number_i64() {
        let l = LooseNumber::<i64>(12);
        assert_de_tokens(&l, &[Token::Str("12")]);
        assert_de_tokens(&l, &[Token::Str("1 2")]);
        assert_de_tokens(&l, &[Token::Str("  1  2   ")]);
        assert_de_tokens(&l, &[Token::Str("  +  1  2   ")]);
        assert_de_tokens(&l, &[Token::Str("12 YEAH")]);
        assert_de_tokens(&l, &[Token::Str("12x222222222222")]);
        let l = LooseNumber::<i64>(-2);
        assert_de_tokens(&l, &[Token::Str("-2")]);
        assert_de_tokens(&l, &[Token::Str("-  2 + ")]);
        assert_de_tokens(&l, &[Token::Str("- 2 - ")]);
        assert_de_tokens(&l, &[Token::Str("-2   FUC ")]);
    }
}
