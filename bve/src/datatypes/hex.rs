use crate::datatypes::*;
use std::fmt;
use std::str::FromStr;

/// A color that is specifically formatted using normal HTML hex notation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HexColorRGB(pub ColorU8RGB);

impl HexColorRGB {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self(ColorU8RGB::new(r, g, b))
    }
}

impl fmt::Display for HexColorRGB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Converting to big endian makes it RRGGBB00, shift to remove the two zeros.
        write!(f, "#{:0>6x}", self.0.as_u32().to_be() >> 8)
    }
}

impl FromStr for HexColorRGB {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let no_hash = &s[1..];
        let number = u32::from_str_radix(no_hash, 16).map_err(drop)?;

        Ok(Self(ColorU8RGB::new(
            ((number >> 16) & 0xFF) as u8,
            ((number >> 8) & 0xFF) as u8,
            (number & 0xFF) as u8,
        )))
    }
}

/// A color with alpha that is specifically formatted using normal HTML hex notation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HexColorRGBA(pub ColorU8RGBA);

impl HexColorRGBA {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(ColorU8RGBA::new(r, g, b, a))
    }
}

impl fmt::Display for HexColorRGBA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:0>8x}", self.0.as_u32().to_be())
    }
}

impl FromStr for HexColorRGBA {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let no_hash = &s[1..];
        let number = u32::from_str_radix(no_hash, 16).map_err(drop)?;

        Ok(Self(ColorU8RGBA::new(
            ((number >> 24) & 0xFF) as u8,
            ((number >> 16) & 0xFF) as u8,
            ((number >> 8) & 0xFF) as u8,
            (number & 0xFF) as u8,
        )))
    }
}

#[cfg(test)]
mod test {
    use crate::{HexColorRGB, HexColorRGBA};
    use std::str::FromStr;

    #[test]
    fn hex_rgb_roundtrip() {
        let input = "#123456";
        let hex_color = HexColorRGB::from_str(input).expect("Parsing failed");
        assert_eq!(hex_color, HexColorRGB::new(0x12, 0x34, 0x56));
        let output = hex_color.to_string();
        assert_eq!(output, "#123456");
    }

    #[test]
    fn hex_rgba_roundtrip() {
        let input = "#12345678";
        let hex_color = HexColorRGBA::from_str(input).expect("Parsing failed");
        assert_eq!(hex_color, HexColorRGBA::new(0x12, 0x34, 0x56, 0x78));
        let output = hex_color.to_string();
        assert_eq!(output, "#12345678");
    }
}
