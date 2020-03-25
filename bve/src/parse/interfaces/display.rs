use crate::parse::util;
use crate::{HexColorRGB, HexColorRGBA};
use cgmath::{Vector2, Vector3, Vector4};
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Display;
use std::io;

/// A display trait for printing BVE Files in a Human Readable format.
///
/// Example of the format:
///
/// ```text
/// ParsedFile:
///     HeaderSection:
///         AString: "Version2.2"
///         AVector3: 1, 2, 3
///     MainSection:
///         AList:
///             0: Value1
///             1: Value1
/// ```
///
/// The following shows which implementation is responsible for which newline or indent.
///
/// - `f1` = `File1`
/// - `s1` = `Section1`
/// - `s2` = `Section2`
/// - `v1` = `Value1`
/// - `v2` = `Value2`
/// - `i1` = `Inner1`
/// - `i1` = `Inner2`
///
/// ```text
/// File1: \f1
/// \f1 Section: \s1
/// \s1 \s1 Value: \v1
/// \v1 \v1 \v1 inner: SomeData \i1
/// \v1 \v1 \v1 inner: SomeData2 \i2
/// \v2 \v2 Value: Out, Out, Out \v2
/// \f1 Section2: \s2
/// \s2 \s2 Value: \v1
/// \v1 \v1 \v1 0: Blah \v1
/// \v1 \v1 \v1 1: Blah \v1
/// ```
///
/// A typical PrettyPrintResult implementation looks like this:
/// 1. If you are the top level object, you need to print your own label (i.e. `ParsedFile:`), and a new line.
/// 2. Depending on the type of object you are, you'll do one of three things:
///   a. If you want to use multiple lines, print a newline.
///   b. If you are using a single line, print the object with the same indent and a newline. You are done.
///   c. If you are delegating to another impl, do not print anything before or after.
/// 3. For every subobject you need to display on their own line:
///   a. Add the indents it needs
///   b. Print its label
///   c. Immediately dispatch to subobject's impl with an increased indent
pub trait PrettyPrintResult {
    /// Prints as is, no indent handling
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()>;

    /// Prints with an indent at the beginning
    fn fmt_indent(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        util::indent(indent, out)?;
        self.fmt(indent, out)
    }
}

macro_rules! debug_impl {
    ($($t:ty),*) => {
        $(
            impl PrettyPrintResult for $t
            {
                fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
                    writeln!(out, "{:?}", self)?;
                    Ok(())
                }
            }
        )*
    };
}

macro_rules! display_impl {
    ($($t:ty),*) => {
        $(
            impl PrettyPrintResult for $t
            {
                fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
                    writeln!(out, "{}", self)?;
                    Ok(())
                }
            }
        )*
    };
}

debug_impl!(str, String);

display_impl!(
    bool,
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
            writeln!(out, "None")?;
        }
        Ok(())
    }
}

impl<T> PrettyPrintResult for Vec<T>
where
    T: PrettyPrintResult,
{
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        if self.is_empty() {
            writeln!(out, "None")?;
        } else {
            writeln!(out)?;
            for (idx, element) in self.iter().enumerate() {
                util::indent(indent, out)?;
                write!(out, "{}:", idx)?;
                element.fmt(indent + 1, out)?;
            }
        }
        Ok(())
    }
}

impl<V> PrettyPrintResult for HashMap<u64, V>
where
    V: PrettyPrintResult,
{
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        if self.is_empty() {
            writeln!(out, "None")?;
        } else {
            writeln!(out)?;
            for (key, value) in self.iter().sorted_by_key(|(k, _)| *k) {
                util::indent(indent, out)?;
                write!(out, "{}: ", key)?;
                value.fmt(indent + 1, out)?;
            }
        }
        Ok(())
    }
}

impl<T> PrettyPrintResult for Vector2<T>
where
    T: Display,
{
    fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        writeln!(out, "{}, {}", self.x, self.y)?;
        Ok(())
    }
}

impl<T> PrettyPrintResult for Vector3<T>
where
    T: Display,
{
    fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        writeln!(out, "{}, {}, {}", self.x, self.y, self.z)?;
        Ok(())
    }
}

impl<T> PrettyPrintResult for Vector4<T>
where
    T: Display,
{
    fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        writeln!(out, "{}, {}, {}, {}", self.x, self.y, self.z, self.w)?;
        Ok(())
    }
}
