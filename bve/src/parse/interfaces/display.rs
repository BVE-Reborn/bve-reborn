use crate::{HexColorRGB, HexColorRGBA};
use cgmath::{Vector2, Vector3};
use itertools::Itertools;
use std::collections::HashMap;
use std::io;

/// A copy of [`fmt::Display`] for BVE's parsers
pub trait PrettyPrintResult {
    /// Prints as is, no indent handling
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()>;

    /// Prints with an indent at the beginning
    fn fmt_indent(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        out.write_all(&vec![b' '; indent * 4])?;
        self.fmt(indent, out)
    }
}

macro_rules! display_impl {
    ($($t:ty),*) => {
        $(
            impl PrettyPrintResult for $t
            {
                fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
                    write!(out, "{}", self)
                }
            }
        )*
    };
}

display_impl!(
    str,
    String,
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    usize,
    f32,
    f64,
    HexColorRGB,
    HexColorRGBA
);

impl<T> PrettyPrintResult for Option<T>
where
    T: PrettyPrintResult,
{
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        if let Some(v) = self {
            v.fmt(indent, out)?;
        } else {
            out.write("None".as_bytes())?;
        }
        Ok(())
    }
}

impl<T> PrettyPrintResult for Vec<T>
where
    T: PrettyPrintResult,
{
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        for element in self {
            element.fmt_indent(indent, out)?;
            out.write(&[b'\n'])?;
        }
        Ok(())
    }
}

impl<K, V> PrettyPrintResult for HashMap<K, V>
where
    K: PrettyPrintResult + Ord,
    V: PrettyPrintResult,
{
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        for (key, value) in self.iter().sorted_by_key(|(k, _)| *k) {
            key.fmt_indent(indent, out)?;
            out.write(" - ".as_bytes())?;
            value.fmt(indent, out)?;
        }
        Ok(())
    }
}

impl<T> PrettyPrintResult for Vector2<T>
where
    T: PrettyPrintResult,
{
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        self.x.fmt(indent, out)?;
        out.write(", ".as_bytes())?;
        self.y.fmt(indent, out)?;
        Ok(())
    }
}

impl<T> PrettyPrintResult for Vector3<T>
where
    T: PrettyPrintResult,
{
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        self.x.fmt(indent, out)?;
        out.write(", ".as_bytes())?;
        self.y.fmt(indent, out)?;
        out.write(", ".as_bytes())?;
        self.z.fmt(indent, out)?;
        Ok(())
    }
}
