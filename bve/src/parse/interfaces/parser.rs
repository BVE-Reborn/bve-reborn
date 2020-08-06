use crate::parse::{
    kvp::{parse_kvp_file, FromKVPFile, KVPSymbols},
    util::strip_comments,
    PrettyPrintResult, UserError,
};
use async_std::path::Path;
use async_trait::async_trait;

/// Types that implement this trait can be parsed from a single input and a way to look up new files.
#[async_trait(?Send)]
pub trait FileAwareFileParser {
    type Output: PrettyPrintResult;
    type Warnings: UserError;
    type Errors: UserError;

    #[must_use]
    async fn file_aware_parse_from<'a, IntoIter, AsRefPath>(
        resolve_bases: IntoIter,
        current_path: &str,
        input: &str,
    ) -> ParserResult<Self::Output, Self::Warnings, Self::Errors>
    where
        IntoIter: IntoIterator<Item = &'a AsRefPath> + Clone + 'a,
        AsRefPath: AsRef<Path> + 'a;
}

#[async_trait(?Send)]
impl<T> FileAwareFileParser for T
where
    T: FileParser,
{
    type Output = <T as FileParser>::Output;
    type Warnings = <T as FileParser>::Warnings;
    type Errors = <T as FileParser>::Errors;

    #[must_use]
    async fn file_aware_parse_from<'a, IntoIter, AsRefPath>(
        _: IntoIter,
        _: &str,
        input: &str,
    ) -> ParserResult<Self::Output, Self::Warnings, Self::Errors>
    where
        IntoIter: IntoIterator<Item = &'a AsRefPath> + Clone + 'a,
        AsRefPath: AsRef<Path> + 'a,
    {
        Self::parse_from(input)
    }
}

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
    type Output = Self;
    type Warnings = <Self as FromKVPFile>::Warnings;
    type Errors = ();

    fn parse_from(input: &str) -> ParserResult<Self::Output, Self::Warnings, Self::Errors> {
        Self::parse_from_kvp(input)
    }
}
