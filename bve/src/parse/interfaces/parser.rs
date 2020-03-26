use crate::parse::{
    kvp::{parse_kvp_file, FromKVPFile, KVPSymbols},
    util::strip_comments,
    PrettyPrintResult, UserError,
};

/// Types that implement this trait can be parsed from a single input.
pub trait FileParser {
    type Output: PrettyPrintResult;
    type Warnings: UserError;
    type Errors: UserError;

    #[must_use]
    fn parse_from(input: &str) -> ParserResult<Self::Output, Self::Warnings, Self::Errors>;
}

/// The result of applying a parser to an input file.
pub struct ParserResult<Output, Warnings, Errors>
where
    Output: PrettyPrintResult,
    Warnings: UserError,
    Errors: UserError,
{
    pub output: Output,
    pub warnings: Vec<Warnings>,
    pub errors: Vec<Errors>,
}

/// A specialized version of [`FileParser`] for parsers based on the KVP parser.
///
/// Only the two constants are needed to correctly implement this.
///
/// Contains a blanket [`FileParser`] impl for all traits that implement this trait.
pub trait KVPFileParser: FromKVPFile + PrettyPrintResult {
    const SYMBOLS: KVPSymbols;
    const COMMENT: char;

    #[must_use]
    fn parse_from_kvp(input: &str) -> ParserResult<Self, Self::Warnings, ()> {
        let lower = strip_comments(input, Self::COMMENT).to_lowercase();
        let kvp_file = parse_kvp_file(&lower, Self::SYMBOLS);

        let (output, warnings) = Self::from_kvp_file(&kvp_file);
        ParserResult {
            output,
            warnings,
            errors: vec![],
        }
    }
}

impl<T> FileParser for T
where
    T: KVPFileParser,
{
    type Errors = ();
    type Output = Self;
    type Warnings = <Self as FromKVPFile>::Warnings;

    fn parse_from(input: &str) -> ParserResult<Self::Output, Self::Warnings, Self::Errors> {
        Self::parse_from_kvp(input)
    }
}
