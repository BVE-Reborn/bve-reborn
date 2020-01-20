#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Span {
    pub line: Option<u64>,
}

impl<'a> From<Option<&'a csv::Position>> for Span {
    #[must_use]
    fn from(p: Option<&'a csv::Position>) -> Self {
        Self {
            line: p.map(csv::Position::line),
        }
    }
}
