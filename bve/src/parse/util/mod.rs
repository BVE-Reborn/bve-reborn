//! Random functions needed for various parts of the parser

pub(in crate::parse) const fn some_zero_u8() -> Option<u8> {
    Some(0)
}

pub(in crate::parse) const fn some_u8_max() -> Option<u8> {
    Some(255)
}

pub(in crate::parse) const fn some_zero_u16() -> Option<u16> {
    Some(0)
}

pub(in crate::parse) const fn some_zero_f32() -> Option<f32> {
    Some(0.0)
}

pub(in crate::parse) const fn some_one_f32() -> Option<f32> {
    Some(1.0)
}

pub(in crate::parse) fn some_string() -> Option<String> {
    Some(String::new())
}
