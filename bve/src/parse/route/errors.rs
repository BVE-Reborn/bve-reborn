use crate::{
    l10n::ForceEnglish,
    localize,
    parse::{UserError, UserErrorCategory},
};
use rand::distributions::WeightedError;
use smallvec::SmallVec;
use smartstring::{LazyCompact, SmartString};

#[derive(Debug, Clone, PartialEq)]
pub enum RouteError {
    PreprocessingError(PreprocessingError),
    ParsingError(SmartString<LazyCompact>),
    CommandCreationError(CommandCreationError),
}

impl UserError for RouteError {
    fn category(&self) -> UserErrorCategory {
        UserErrorCategory::Error
    }

    fn line(&self) -> Option<u64> {
        None
    }

    fn description(&self, en: ForceEnglish) -> String {
        match self {
            Self::PreprocessingError(postprocessing_error) => match postprocessing_error {
                PreprocessingError::MalformedDirective { directive } => {
                    localize!(@en, "route-preprocessing-malformed-directive", "directive" -> directive.as_str())
                }
                PreprocessingError::IncludeFileNotFound { file } => {
                    localize!(@en, "route-preprocessing-include-file-not-found", "file" -> file.as_str())
                }
                PreprocessingError::RandomIncludeError {
                    weights,
                    sub: weighted_error,
                } => match weighted_error {
                    WeightedError::NoItem => localize!(@en, "route-preprocessing-random-include-none"),
                    WeightedError::InvalidWeight => {
                        let weights_string = format!("{:?}", weights);
                        localize!(@en, "route-preprocessing-random-invalid-weight", "weights" -> weights_string.as_str())
                    }
                    WeightedError::AllWeightsZero => localize!(@en, "route-preprocessing-random-all-zero"),
                    WeightedError::TooMany => unreachable!("Should OOM before we get here :)"),
                },
                PreprocessingError::InvalidChrArgument { code } => {
                    localize!(@en, "route-preprocessing-invalid-argument", "arg" -> code.as_str(), "directive" -> "chr")
                }
                PreprocessingError::InvalidSubArgument { code } => {
                    localize!(@en, "route-preprocessing-invalid-argument", "arg" -> code.as_str(), "directive" -> "sub")
                }
            },
            Self::ParsingError(command) => {
                localize!(@en, "route-preprocessing-invalid-argument", "command" -> command.as_str())
            }
            _ => unimplemented!(),
        }
    }
}

impl From<PreprocessingError> for RouteError {
    fn from(err: PreprocessingError) -> Self {
        RouteError::PreprocessingError(err)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreprocessingError {
    /// Directive syntax is incorrect
    MalformedDirective { directive: SmartString<LazyCompact> },
    /// Invalid ascii code
    InvalidChrArgument { code: SmartString<LazyCompact> },
    /// Invalid integer for sub command
    InvalidSubArgument { code: SmartString<LazyCompact> },
    /// Include file doesn't exist
    IncludeFileNotFound { file: SmartString<LazyCompact> },
    /// Invalid random include.
    RandomIncludeError {
        weights: SmallVec<[i64; 8]>,
        sub: WeightedError,
    },
}

impl From<CommandCreationError> for RouteError {
    fn from(err: CommandCreationError) -> Self {
        RouteError::CommandCreationError(err)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandCreationError {
    /// No namespace with a command that needs it
    MissingNamespace { command: String },
    /// Missing required index
    MissingIndex { command: String, index: usize },
    /// Index cannot be parsed
    InvalidIndex { command: String, index: usize },
    /// Missing required argument
    MissingArgument { command: String, index: usize },
    /// Argument cannot be parsed
    InvalidArgument { command: String, index: usize },
    /// Suffix is missing
    MissingSuffix { command: String },
    /// Suffix is missing
    InvalidSuffix { command: String },
    /// Namespace/command/suffix combination is unknown
    UnknownCommand {
        namespace: SmartString<LazyCompact>,
        command: SmartString<LazyCompact>,
        suffix: Option<SmartString<LazyCompact>>,
    },
}
