use crate::{
    filesystem,
    parse::{FileAwareFileParser, ParserResult, PrettyPrintResult},
};
use async_std::path::Path;
use async_trait::async_trait;
use rand::SeedableRng;
use smallvec::SmallVec;
use std::{cell::RefCell, io};

pub mod errors;
pub mod ir;
pub mod parser;
pub mod preprocessor;

pub type TrackPositionSmallVec = SmallVec<[f32; 4]>;

#[derive(Debug)]
pub struct ParsedRoute(Vec<ir::ParsedDirective>);

impl PrettyPrintResult for ParsedRoute {
    fn fmt(&self, _indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        write!(out, "{:#?}", self)
    }
}

#[async_trait(?Send)]
impl FileAwareFileParser for ParsedRoute {
    type Output = Self;
    type Warnings = ();
    type Errors = errors::RouteError;

    async fn file_aware_parse_from<'a, IntoIter, AsRefPath>(
        resolve_bases: IntoIter,
        current_path: &str,
        input: &str,
    ) -> ParserResult<Self::Output, Self::Warnings, Self::Errors>
    where
        IntoIter: IntoIterator<Item = &'a AsRefPath> + Clone + 'a,
        AsRefPath: AsRef<Path> + ?Sized + 'a,
    {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let resolve_bases_ref = &resolve_bases;
        let file_func = |input: preprocessor::FileInput| async move {
            let current_dir = Path::new(&input.base_path).parent().expect("Path has no parent");
            try {
                let requested_path = &*input.requested_path;
                #[allow(clippy::redundant_closure_for_method_calls)] // needed for the manual lifetime
                let file = filesystem::resolve_path_bases(
                    resolve_bases_ref
                        .clone()
                        .into_iter()
                        .map(|v: &'a AsRefPath| v.as_ref())
                        .chain(std::iter::once(current_dir)),
                    &input.requested_path,
                )
                .await
                .ok_or_else(|| errors::PreprocessingError::IncludeFileNotFound {
                    file: requested_path.into(),
                })?;
                let contents = async_std::fs::read_to_string(&file).await.map_err(|error| {
                    errors::PreprocessingError::IncludeFileUnreadable {
                        file: requested_path.into(),
                        error,
                    }
                })?;
                preprocessor::FileOutput {
                    path: file.to_string_lossy().to_string(),
                    contents,
                }
            }
        };
        let (preprocessed, errors) = preprocessor::preprocess_route(current_path, input, &mut rng, file_func).await;
        let error_refcell = RefCell::new(errors);
        let parsed = parser::parse_route(&preprocessed, &error_refcell);
        let commands = ir::CommandParserIterator::new(parsed, &error_refcell);
        ParserResult {
            output: ParsedRoute(commands.collect()),
            warnings: Vec::new(),
            errors: error_refcell.into_inner(),
        }
    }
}
