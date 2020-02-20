use crate::parse::kvp::{KVPField, KVPFile, KVPSection};
use cgmath::{Vector1, Vector2, Vector3, Vector4};
use std::str::FromStr;

pub trait FromKVPFile: Default {
    fn from_kvp_file(k: &KVPFile<'_>) -> Self;
}

pub trait FromKVPSection: Default {
    fn from_kvp_section(section: &KVPSection<'_>) -> Self;
}

pub trait FromKVPValue {
    fn from_kvp_value(value: &str) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! impl_from_kvp_value_primative {
    ($($int:ident),+) => {$(
        impl FromKVPValue for $int
        {
            fn from_kvp_value(value: &str) -> Option<$int> {
                $int::from_str(value).ok()
            }
        }
    )*};
}

impl_from_kvp_value_primative!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64);

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
        Some(String::from(value))
    }
}

impl<T> FromKVPValue for Vec<T>
where
    T: FromKVPValue + Default,
{
    fn from_kvp_value(value: &str) -> Option<Self> {
        let split = value.split(",").map(str::trim);
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
        let split: Vec<&str> = value.split(",").map(str::trim).collect();
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
        let split: Vec<&str> = value.split(",").map(str::trim).collect();
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
        let split: Vec<&str> = value.split(",").map(str::trim).collect();
        Some(Self::new(
            split.get(0).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
            split.get(1).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
            split.get(2).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
            split.get(3).and_then(|v| T::from_kvp_value(v)).unwrap_or_default(),
        ))
    }
}
