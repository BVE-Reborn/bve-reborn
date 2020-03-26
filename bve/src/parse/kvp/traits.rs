use crate::{
    l10n::ForceEnglish,
    localize,
    parse::{
        kvp::{KVPFile, KVPSection},
        util::{parse_loose_number, parse_loose_numeric_bool},
        Span, UserError, UserErrorCategory,
    },
    HexColorRGB, HexColorRGBA,
};
use cgmath::{Vector1, Vector2, Vector3, Vector4};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct KVPGenericWarning {
    pub span: Span,
    pub kind: KVPGenericWarningKind,
}

impl UserError for KVPGenericWarning {
    fn category(&self) -> UserErrorCategory {
        UserErrorCategory::Warning
    }

    fn line(&self) -> u64 {
        self.span.line.unwrap_or(0)
    }

    fn description(&self, en: ForceEnglish) -> String {
        match &self.kind {
            KVPGenericWarningKind::UnknownSection { name } => {
                localize!(@en, "kvp-unknown-section", "section" -> name.as_str())
            }
            KVPGenericWarningKind::UnknownField { name } => {
                localize!(@en, "kvp-unknown-field", "field" -> name.as_str())
            }
            KVPGenericWarningKind::TooManyFields { idx, max } => {
                localize!(@en, "kvp-too-many-fields", "number" -> idx + 1, "total" -> max)
            }
            KVPGenericWarningKind::InvalidValue { value } => {
                localize!(@en, "kvp-invalid-value", "value" -> value.as_str())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KVPGenericWarningKind {
    UnknownSection { name: String },
    UnknownField { name: String },
    TooManyFields { idx: u64, max: u64 },
    InvalidValue { value: String },
}

pub trait FromKVPFile: Default {
    type Warnings: UserError;
    #[must_use]
    fn from_kvp_file(k: &KVPFile<'_>) -> (Self, Vec<Self::Warnings>);
}

pub trait FromKVPSection: Default {
    type Warnings: UserError;
    #[must_use]
    fn from_kvp_section(section: &KVPSection<'_>) -> (Self, Vec<Self::Warnings>);
}

impl<T> FromKVPSection for Option<T>
where
    T: FromKVPSection,
{
    type Warnings = T::Warnings;

    fn from_kvp_section(section: &KVPSection<'_>) -> (Self, Vec<Self::Warnings>) {
        let (o, e) = T::from_kvp_section(section);
        (Some(o), e)
    }
}

pub trait FromKVPValue {
    #[must_use]
    fn from_kvp_value(value: &str) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! impl_from_kvp_value_primitive {
    ($($prim:ident),+) => {$(
        impl FromKVPValue for $prim
        {
            fn from_kvp_value(value: &str) -> Option<$prim> {
                parse_loose_number::<$prim>(value).map(|v| v.0)
            }
        }
    )*};
}

impl_from_kvp_value_primitive!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64);

impl FromKVPValue for bool {
    fn from_kvp_value(value: &str) -> Option<Self> {
        parse_loose_numeric_bool(value).map(|v| v.0)
    }
}

impl<T> FromKVPValue for Option<T>
where
    T: FromKVPValue,
{
    fn from_kvp_value(value: &str) -> Option<Self> {
        Some(T::from_kvp_value(value))
    }
}

impl FromKVPValue for String {
    fn from_kvp_value(value: &str) -> Option<Self> {
        Some(Self::from(value))
    }
}

impl<T> FromKVPValue for Vec<T>
where
    T: FromKVPValue + Default,
{
    fn from_kvp_value(value: &str) -> Option<Self> {
        let split = value.split(',').map(str::trim);
        Some(split.map(T::from_kvp_value).map(Option::unwrap_or_default).collect())
    }
}

impl<T> FromKVPValue for Vector1<T>
where
    T: FromKVPValue + Default,
{
    fn from_kvp_value(value: &str) -> Option<Self> {
        Some(Self::new(T::from_kvp_value(value).unwrap_or_default()))
    }
}

impl<T> FromKVPValue for Vector2<T>
where
    T: FromKVPValue + Default,
{
    fn from_kvp_value(value: &str) -> Option<Self> {
        let split: Vec<&str> = value.split(',').map(str::trim).collect();
        Some(Self::new(
            split.get(0).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
            split.get(1).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
        ))
    }
}

impl<T> FromKVPValue for Vector3<T>
where
    T: FromKVPValue + Default,
{
    fn from_kvp_value(value: &str) -> Option<Self> {
        let split: Vec<&str> = value.split(',').map(str::trim).collect();
        Some(Self::new(
            split.get(0).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
            split.get(1).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
            split.get(2).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
        ))
    }
}

impl<T> FromKVPValue for Vector4<T>
where
    T: FromKVPValue + Default,
{
    fn from_kvp_value(value: &str) -> Option<Self> {
        let split: Vec<&str> = value.split(',').map(str::trim).collect();
        Some(Self::new(
            split.get(0).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
            split.get(1).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
            split.get(2).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
            split.get(3).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
        ))
    }
}

impl FromKVPValue for HexColorRGB {
    fn from_kvp_value(value: &str) -> Option<Self> {
        Self::from_str(value).ok()
    }
}

impl FromKVPValue for HexColorRGBA {
    fn from_kvp_value(value: &str) -> Option<Self> {
        Self::from_str(value).ok()
    }
}
