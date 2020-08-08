use crate::parse::route::{
    errors::{PreprocessingError, RouteError},
    TrackPositionSmallVec,
};
use bve_common::nom::{separated_list_small, w, MapOutput};
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag_no_case, take_while1},
    combinator::{map_res, opt},
    sequence::{delimited, preceded, separated_pair},
    IResult,
};
use once_cell::sync::Lazy;
use regex::Regex;
use smallvec::SmallVec;
use smartstring::{LazyCompact, SmartString};
use std::{
    cell::RefCell,
    convert::{identity, TryFrom},
    fmt,
    str::FromStr,
};

static COMMAND_SPLIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[,\r\n]").expect("invalid regex"));
static CHR_APPLY_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"%C([-\d]+)%").expect("invalid regex"));

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub namespace: Option<SmartString<LazyCompact>>,
    pub name: SmartString<LazyCompact>,
    pub indices: IndexSmallVec,
    pub suffix: Option<SmartString<LazyCompact>>,
    pub arguments: ArgumentSmallVec,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref namespace) = self.namespace {
            write!(f, "{}", namespace)?;
        }
        write!(f, ".{}", self.name)?;
        if !self.indices.is_empty() {
            write!(
                f,
                "({})",
                self.indices
                    .iter()
                    .map(|v| v.map(|i| i.to_string()).unwrap_or_else(String::new))
                    .join("; ")
            )?;
        }
        if let Some(ref suffix) = self.suffix {
            write!(f, ".{}", suffix)?;
        }
        if !self.arguments.is_empty() {
            write!(f, " {}", &self.arguments.join("; "))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // We never have more than one of these, so being large is fine.
pub enum Directive {
    TrackPosition(TrackPositionSmallVec),
    Command(Command),
    With(SmartString<LazyCompact>),
    TrackPositionOffset(f32),
}

pub type IndexSmallVec = SmallVec<[Option<i64>; 8]>;
pub type ArgumentSmallVec = SmallVec<[SmartString<LazyCompact>; 8]>;

pub fn parse_route<'a>(
    preprocessed: &'a str,
    errors: &'a RefCell<Vec<RouteError>>,
) -> impl Iterator<Item = Directive> + 'a {
    split_into_commands(preprocessed)
        .filter_map(move |v| match apply_chr(v) {
            Ok(r) => Some(r),
            Err(err) => {
                errors.borrow_mut().push(err.into());
                None
            }
        })
        .filter_map(remove_comments)
        .filter_map(move |v| match parse_directive(&v) {
            Some(r) => Some(r),
            None => {
                errors.borrow_mut().push(RouteError::Parsing(v));
                None
            }
        })
}

fn split_into_commands(input: &str) -> impl Iterator<Item = &str> {
    COMMAND_SPLIT_REGEX
        .split(input)
        .map(str::trim)
        .filter(|&s| !s.is_empty())
}

fn apply_chr(input: &str) -> Result<SmartString<LazyCompact>, PreprocessingError> {
    let mut output = SmartString::new();
    let mut last_capture = 0_usize;
    for capture in CHR_APPLY_REGEX.captures_iter(input) {
        let mat = capture.get(0).unwrap_or_else(|| unreachable!());
        output.push_str(&input[last_capture..mat.start()]);

        let number_str = capture.get(1).unwrap_or_else(|| unreachable!()).as_str();
        let number: u32 = number_str.parse().map_err(|_| PreprocessingError::InvalidChrArgument {
            code: number_str.into(),
        })?;
        output.push(
            char::try_from(number).map_err(|_| PreprocessingError::InvalidChrArgument {
                code: number_str.into(),
            })?,
        );

        last_capture = mat.end();
    }
    output.push_str(&input[last_capture..]);

    output = SmartString::from(output.trim());
    Ok(output)
}

fn remove_comments(input: SmartString<LazyCompact>) -> Option<SmartString<LazyCompact>> {
    if input.chars().next()? == ';' {
        None
    } else {
        Some(input)
    }
}

fn parse_directive(command: &str) -> Option<Directive> {
    alt((
        parse_with_offset,
        parse_with,
        parse_track_position,
        parse_command_indices_args,
        parse_command_args,
        parse_command,
    ))(command)
    .ok()
    .and_then(|(input, directive)| if input.is_empty() { Some(directive) } else { None })
}

fn parse_command(command: &str) -> IResult<&str, Directive> {
    let (command, (namespace, name)) =
        separated_pair(opt(w(parse_identifier)), w(tag_no_case(".")), parse_identifier)(command)?;
    Ok((
        command,
        Directive::Command(Command {
            namespace: namespace.map(SmartString::from),
            name: SmartString::from(name),
            indices: SmallVec::new(),
            suffix: None,
            arguments: SmallVec::new(),
        }),
    ))
}

fn parse_command_args(command: &str) -> IResult<&str, Directive> {
    let (command, (namespace, name)) =
        separated_pair(opt(w(parse_identifier)), w(tag_no_case(".")), parse_identifier)(command)?;
    let (command, arguments) = alt((
        delimited(w(tag_no_case("(")), parse_argument_list, opt(w(tag_no_case(")")))),
        parse_argument_list_free,
    ))(command)?;
    Ok((
        command,
        Directive::Command(Command {
            namespace: namespace.map(SmartString::from),
            name: SmartString::from(name),
            indices: SmallVec::new(),
            suffix: None,
            arguments,
        }),
    ))
}

fn parse_command_indices_args(command: &str) -> IResult<&str, Directive> {
    let (command, (namespace, name)) =
        match separated_pair(opt(w(parse_identifier)), w(tag_no_case(".")), parse_identifier)(command) {
            Ok(v) => v,
            Err(_) => {
                let (command, name) = w(tag_no_case("signal"))(command)?;
                (command, (Some("signal"), name))
            }
        };
    let (command, indices) = delimited(w(tag_no_case("(")), parse_indices, w(tag_no_case(")")))(command)?;
    let (command, suffix) = opt(preceded(w(tag_no_case(".")), parse_identifier))(command)?;
    let (command, suffix2) = if suffix.is_some() {
        opt(preceded(w(tag_no_case(".")), parse_identifier))(command)?
    } else {
        (command, None)
    };
    let (command, arguments) = if suffix.is_some() {
        alt((
            delimited(w(tag_no_case("(")), parse_argument_list1, opt(w(tag_no_case(")")))),
            parse_argument_list1_free,
        ))(command)?
    } else {
        parse_argument_list1_free(command)?
    };
    Ok((
        command,
        Directive::Command(Command {
            namespace: namespace.map(SmartString::from),
            name: SmartString::from(name),
            indices,
            suffix: suffix.or(suffix2).map(SmartString::from),
            arguments,
        }),
    ))
}

fn parse_with(command: &str) -> IResult<&str, Directive> {
    preceded(w(tag_no_case("with")), parse_identifier)(command).map_output(|v| Directive::With(SmartString::from(v)))
}

fn parse_with_offset(command: &str) -> IResult<&str, Directive> {
    map_res(
        delimited(w(tag_no_case("%O")), parse_floating_number, w(tag_no_case("%"))),
        |v| v.map(Directive::TrackPositionOffset).ok_or(()),
    )(command)
}

fn parse_track_position(command: &str) -> IResult<&str, Directive> {
    map_res(
        separated_list_small(w(tag_no_case(";")), parse_floating_number),
        |list| {
            if list.is_empty() { Err(()) } else { Ok(list) }
        },
    )(command)
    .map_output(|array: SmallVec<[Option<f32>; 8]>| {
        Directive::TrackPosition(array.into_iter().filter_map(identity).collect())
    })
}

fn parse_indices(command: &str) -> IResult<&str, IndexSmallVec> {
    map_res(
        separated_list_small(w(tag_no_case(";")), parse_integer_number),
        |list: SmallVec<[Option<i64>; 8]>| {
            if list.is_empty() { Err(()) } else { Ok(list) }
        },
    )(command)
    .map_output(|array: SmallVec<[Option<i64>; 8]>| array.into_iter().collect())
}

// Returns None if matched nothing
fn parse_floating_number(command: &str) -> IResult<&str, Option<f32>> {
    map_res::<_, _, _, _, <f32 as FromStr>::Err, _, _>(opt(w(is_a("0123456789.-E"))), |s: Option<&str>| {
        if let Some(inner) = s {
            Ok(Some(inner.parse()?))
        } else {
            Ok(None)
        }
    })(command)
}

// Returns None if matched nothing
fn parse_integer_number(command: &str) -> IResult<&str, Option<i64>> {
    map_res::<_, _, _, _, <i64 as FromStr>::Err, _, _>(opt(w(is_a("0123456789-"))), |s: Option<&str>| {
        if let Some(inner) = s {
            Ok(Some(inner.parse()?))
        } else {
            Ok(None)
        }
    })(command)
}

fn parse_identifier(command: &str) -> IResult<&str, &str> {
    w(take_while1(char::is_alphabetic))(command)
}

fn parse_argument_list1(command: &str) -> IResult<&str, ArgumentSmallVec> {
    map_res(
        parse_argument_list,
        |list| {
            if list.is_empty() { Err(()) } else { Ok(list) }
        },
    )(command)
}

fn parse_argument_list1_free(command: &str) -> IResult<&str, ArgumentSmallVec> {
    map_res(parse_argument_list_free, |list| {
        if list.is_empty() { Err(()) } else { Ok(list) }
    })(command)
}

fn parse_argument_list(command: &str) -> IResult<&str, ArgumentSmallVec> {
    separated_list_small(w(tag_no_case(";")), parse_argument)(command).map_output(
        |array: SmallVec<[Option<&str>; 8]>| {
            array
                .into_iter()
                .map(|arg| arg.map_or_else(SmartString::new, SmartString::from))
                .collect()
        },
    )
}

fn parse_argument_list_free(command: &str) -> IResult<&str, ArgumentSmallVec> {
    let (command, _) = opt(w(tag_no_case(";")))(command)?;
    separated_list_small(w(tag_no_case(";")), parse_argument_free)(command).map_output(
        |array: SmallVec<[Option<&str>; 8]>| {
            array
                .into_iter()
                .map(|arg| arg.map_or_else(SmartString::new, SmartString::from))
                .collect()
        },
    )
}

fn parse_argument(command: &str) -> IResult<&str, Option<&str>> {
    opt(w(is_not("(;)")))(command).map_output(|opt| opt.map(str::trim))
}

fn parse_argument_free(command: &str) -> IResult<&str, Option<&str>> {
    opt(w(is_not(";")))(command).map_output(|opt| opt.map(str::trim))
}

#[cfg(test)]
mod test {
    use super::*;

    fn default_command() -> Command {
        Command {
            namespace: None,
            name: SmartString::new(),
            indices: SmallVec::default(),
            suffix: None,
            arguments: SmallVec::default(),
        }
    }

    macro_rules! ss {
        ($str:literal) => {
            SmartString::from($str)
        };
    }

    #[test]
    fn with_statement() {
        assert_eq!(parse_directive("With Blob"), Some(Directive::With(ss!("Blob"))));
        assert_eq!(parse_directive("With    BlobH "), Some(Directive::With(ss!("BlobH"))));
        assert_eq!(parse_directive("With"), None);
    }

    macro_rules! smallvec_opt {
        ($($value:expr),*) => {smallvec::smallvec![$(Some($value)),*]};
    }

    #[test]
    fn track_position() {
        assert_eq!(
            parse_directive("1000"),
            Some(Directive::TrackPosition(smallvec::smallvec![1000.0]))
        );
        assert_eq!(
            parse_directive("1000 ;;; ; ; ;; ;"),
            Some(Directive::TrackPosition(smallvec::smallvec![1000.0]))
        );
        assert_eq!(
            parse_directive("1000;2000"),
            Some(Directive::TrackPosition(smallvec::smallvec![1000.0, 2000.0]))
        );
        assert_eq!(
            parse_directive("1000  ; 2000"),
            Some(Directive::TrackPosition(smallvec::smallvec![1000.0, 2000.0]))
        );
        assert_eq!(
            parse_directive("1000.42;2000.84"),
            Some(Directive::TrackPosition(smallvec::smallvec![1000.42, 2000.84]))
        );
        assert_eq!(parse_directive(""), None);
        assert_eq!(parse_directive(";"), None);
    }

    #[test]
    fn offset_marker() {
        assert_eq!(parse_directive("%O10%"), Some(Directive::TrackPositionOffset(10.0)));
        assert_eq!(parse_directive("%O-10%"), Some(Directive::TrackPositionOffset(-10.0)));
        assert_eq!(parse_directive("%O-10.2%"), Some(Directive::TrackPositionOffset(-10.2)));
    }

    #[test]
    fn command() {
        assert_eq!(
            parse_directive(".command"),
            Some(Directive::Command(Command {
                name: ss!("command"),
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive("  .  command  "),
            Some(Directive::Command(Command {
                name: ss!("command"),
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive("namespace.command"),
            Some(Directive::Command(Command {
                namespace: Some(ss!("namespace")),
                name: ss!("command"),
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive("  namespace .  command  "),
            Some(Directive::Command(Command {
                namespace: Some(ss!("namespace")),
                name: ss!("command"),
                ..default_command()
            }))
        );
    }

    #[test]
    fn command_arguments() {
        assert_eq!(
            parse_directive(".command( a 1 ; b 2 ; c 3 ; ; ; )"),
            Some(Directive::Command(Command {
                name: ss!("command"),
                arguments: smallvec::smallvec![ss!("a 1"), ss!("b 2"), ss!("c 3"), ss!(""), ss!(""), ss!("")],
                ..default_command()
            }))
        );
        // makes sure this isn't mistaken for indices
        assert_eq!(
            parse_directive(".command(0)"),
            Some(Directive::Command(Command {
                name: ss!("command"),
                arguments: smallvec::smallvec![ss!("0")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive("f  .  command   ( a 1 ; b 2 ; c 3 )  "),
            Some(Directive::Command(Command {
                namespace: Some(ss!("f")),
                name: ss!("command"),
                arguments: smallvec::smallvec![ss!("a 1"), ss!("b 2"), ss!("c 3")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(".command a 1 ; b 2 ; c 3 "),
            Some(Directive::Command(Command {
                name: ss!("command"),
                arguments: smallvec::smallvec![ss!("a 1"), ss!("b 2"), ss!("c 3")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(".command a 1 ; b 2 ; c 3 ; ; ; "),
            Some(Directive::Command(Command {
                name: ss!("command"),
                arguments: smallvec::smallvec![ss!("a 1"), ss!("b 2"), ss!("c 3"), ss!(""), ss!(""), ss!("")],
                ..default_command()
            }))
        );
    }

    #[test]
    fn command_indices() {
        assert_eq!(
            parse_directive(".command(-1;2;3;1) f"),
            Some(Directive::Command(Command {
                name: ss!("command"),
                indices: smallvec_opt![-1, 2, 3, 1],
                arguments: smallvec::smallvec![ss!("f")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(".command(-1;2;3;1;;) f;; ;;"),
            Some(Directive::Command(Command {
                name: ss!("command"),
                indices: smallvec::smallvec![Some(-1), Some(2), Some(3), Some(1), None, None],
                arguments: smallvec::smallvec![ss!("f"), ss!(""), ss!(""), ss!(""), ss!("")],
                ..default_command()
            }))
        );
    }

    #[test]
    fn command_suffix() {
        assert_eq!(
            parse_directive(".command(-1;2;3;1).h f"),
            Some(Directive::Command(Command {
                name: ss!("command"),
                indices: smallvec_opt![-1, 2, 3, 1],
                suffix: Some(ss!("h")),
                arguments: smallvec::smallvec![ss!("f")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(".command(-1;2;3;1).h(f)"),
            Some(Directive::Command(Command {
                name: ss!("command"),
                indices: smallvec_opt![-1, 2, 3, 1],
                suffix: Some(ss!("h")),
                arguments: smallvec::smallvec![ss!("f")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(".command(-1;2;3;1).h.j f"),
            Some(Directive::Command(Command {
                name: ss!("command"),
                indices: smallvec_opt![-1, 2, 3, 1],
                suffix: Some(ss!("h")),
                arguments: smallvec::smallvec![ss!("f")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive("namespace  . command (-1;2;3;1) . h (f;f2; 3;;)"),
            Some(Directive::Command(Command {
                namespace: Some(ss!("namespace")),
                name: ss!("command"),
                indices: smallvec_opt![-1, 2, 3, 1],
                suffix: Some(ss!("h")),
                arguments: smallvec::smallvec![ss!("f"), ss!("f2"), ss!("3"), ss!(""), ss!("")],
            }))
        );
    }

    #[test]
    fn signal_command() {
        assert_eq!(
            parse_directive("signal(2).Load H; K"),
            Some(Directive::Command(Command {
                namespace: Some(ss!("signal")),
                name: ss!("signal"),
                indices: smallvec_opt![2],
                suffix: Some(ss!("Load")),
                arguments: smallvec::smallvec![ss!("H"), ss!("K")],
            }))
        );
    }

    #[test]
    fn parens_command() {
        assert_eq!(
            parse_directive(".WallL(7) some_path_with(parens)"),
            Some(Directive::Command(Command {
                name: ss!("WallL"),
                indices: smallvec_opt![7],
                arguments: smallvec::smallvec![ss!("some_path_with(parens)")],
                ..default_command()
            }))
        );
    }

    #[test]
    fn extra_semicolon() {
        assert_eq!(
            parse_directive(".Boop ;boop"),
            Some(Directive::Command(Command {
                name: ss!("Boop"),
                arguments: smallvec::smallvec![ss!("boop")],
                ..default_command()
            }))
        );
    }
}
