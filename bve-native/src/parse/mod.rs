//! C interface for [`bve::parse`].

use crate::COption;

pub mod mesh;

/// C safe wrapper for [`Span`](bve::parse::Span).
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Span {
    pub line: COption<u64>,
}

impl From<bve::parse::Span> for Span {
    fn from(other: bve::parse::Span) -> Self {
        Self {
            line: other.line.into(),
        }
    }
}

impl Into<bve::parse::Span> for Span {
    fn into(self) -> bve::parse::Span {
        bve::parse::Span { line: self.line.into() }
    }
}
