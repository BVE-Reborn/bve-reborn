use num_traits::ToPrimitive;

/// File location for errors/ast nodes
///
/// Does not contain file information because they are already associated with an attempt to parse a file.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    /// Line of the location
    ///
    /// May be empty if there is no reasonable way to create a span for a construct.
    /// Empty spans generally should not be exposed to the user.
    pub line: Option<u64>,
}

impl Span {
    pub fn new() -> Self {
        Self { line: None }
    }

    pub fn none() -> Self {
        Self { line: None }
    }

    pub fn from_line(line: impl ToPrimitive) -> Self {
        Self { line: line.to_u64() }
    }
}

impl<'a> From<Option<&'a csv::Position>> for Span {
    #[must_use]
    fn from(p: Option<&'a csv::Position>) -> Self {
        Self {
            line: p.map(csv::Position::line),
        }
    }
}
