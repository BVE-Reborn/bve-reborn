use bve_common::nom::{separated_list_small, w, MapOutput};
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
use std::{convert::identity, str::FromStr};

static COMMAND_SPLIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[,\r\n]").expect("invalid regex"));

#[derive(Debug, Clone, PartialEq)]
pub enum Directive<'a> {
    TrackPosition(TrackPositionSmallVec),
    Command {
        namespace: Option<&'a str>,
        name: &'a str,
        indices: IndexSmallVec,
        suffix: Option<&'a str>,
        arguments: ArgumentSmallVec<'a>,
    },
    With(&'a str),
}

type TrackPositionSmallVec = SmallVec<[f32; 4]>;
type IndexSmallVec = SmallVec<[i64; 8]>;
type ArgumentSmallVec<'a> = SmallVec<[&'a str; 8]>;

pub fn parse_route(preprocessed: &str) -> impl Iterator<Item = Directive<'_>> {
    split_into_commands(preprocessed).filter_map(parse_directive)
}

fn split_into_commands(input: &str) -> impl Iterator<Item = &str> {
    COMMAND_SPLIT_REGEX
        .split(input)
        .map(str::trim)
        .filter(|&s| !s.is_empty())
}

fn parse_directive(command: &str) -> Option<Directive<'_>> {
    alt((
        parse_with,
        parse_track_position,
        parse_command_indices_args,
        parse_command_args,
        parse_command,
    ))(command)
    .ok()
    .and_then(|(input, directive)| if input.is_empty() { Some(directive) } else { None })
}

fn parse_command(command: &str) -> IResult<&str, Directive<'_>> {
    let (command, (namespace, name)) =
        separated_pair(opt(w(parse_identifier)), w(tag_no_case(".")), parse_identifier)(command)?;
    Ok((command, Directive::Command {
        namespace,
        name,
        indices: SmallVec::new(),
        suffix: None,
        arguments: SmallVec::new(),
    }))
}

fn parse_command_args(command: &str) -> IResult<&str, Directive<'_>> {
    let (command, (namespace, name)) =
        separated_pair(opt(w(parse_identifier)), w(tag_no_case(".")), parse_identifier)(command)?;
    let (command, arguments) = alt((
        delimited(w(tag_no_case("(")), parse_argument_list, opt(w(tag_no_case(")")))),
        parse_argument_list,
    ))(command)?;
    Ok((command, Directive::Command {
        namespace,
        name,
        indices: SmallVec::new(),
        suffix: None,
        arguments,
    }))
}

fn parse_command_indices_args(command: &str) -> IResult<&str, Directive<'_>> {
    let (command, (namespace, name)) =
        separated_pair(opt(w(parse_identifier)), w(tag_no_case(".")), parse_identifier)(command)?;
    let (command, indices) = delimited(w(tag_no_case("(")), parse_indices, w(tag_no_case(")")))(command)?;
    let (command, suffix) = opt(preceded(w(tag_no_case(".")), parse_identifier))(command)?;
    let (command, arguments) = if suffix.is_some() {
        alt((
            delimited(w(tag_no_case("(")), parse_argument_list1, opt(w(tag_no_case(")")))),
            parse_argument_list1,
        ))(command)?
    } else {
        parse_argument_list1(command)?
    };
    Ok((command, Directive::Command {
        namespace,
        name,
        indices,
        suffix,
        arguments,
    }))
}

fn parse_with(command: &str) -> IResult<&str, Directive<'_>> {
    preceded(w(tag_no_case("with")), parse_identifier)(command).map_output(|v| Directive::With(v))
}

fn parse_track_position(command: &str) -> IResult<&str, Directive<'_>> {
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
    .map_output(|array: SmallVec<[Option<i64>; 8]>| array.into_iter().filter_map(identity).collect())
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

fn parse_argument_list1(command: &str) -> IResult<&str, ArgumentSmallVec<'_>> {
    map_res(
        parse_argument_list,
        |list| {
            if list.is_empty() { Err(()) } else { Ok(list) }
        },
    )(command)
}

fn parse_argument_list(command: &str) -> IResult<&str, ArgumentSmallVec<'_>> {
    separated_list_small(w(tag_no_case(";")), parse_argument)(command).map_output(
        |array: SmallVec<[Option<&str>; 8]>| {
            array
                .into_iter()
                .filter(|arg| arg.map(|v| !v.is_empty()).unwrap_or(false))
                .map(|arg| arg.unwrap_or_else(|| unreachable!()))
                .collect()
        },
    )
}

fn parse_argument(command: &str) -> IResult<&str, Option<&str>> {
    opt(w(is_not("(;)")))(command).map_output(|opt| opt.map(str::trim))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn with_statement() {
        assert_eq!(parse_directive("With Blob"), Some(Directive::With("Blob")));
        assert_eq!(parse_directive("With    BlobH "), Some(Directive::With("BlobH")));
        assert_eq!(parse_directive("With"), None);
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
    fn command() {
        assert_eq!(
            parse_directive(".command"),
            Some(Directive::Command {
                namespace: None,
                name: "command",
                indices: SmallVec::new(),
                suffix: None,
                arguments: SmallVec::new(),
            })
        );
        assert_eq!(
            parse_directive("  .  command  "),
            Some(Directive::Command {
                namespace: None,
                name: "command",
                indices: SmallVec::new(),
                suffix: None,
                arguments: SmallVec::new(),
            })
        );
        assert_eq!(
            parse_directive("namespace.command"),
            Some(Directive::Command {
                namespace: Some("namespace"),
                name: "command",
                indices: SmallVec::new(),
                suffix: None,
                arguments: SmallVec::new(),
            })
        );
        assert_eq!(
            parse_directive("  namespace .  command  "),
            Some(Directive::Command {
                namespace: Some("namespace"),
                name: "command",
                indices: SmallVec::new(),
                suffix: None,
                arguments: SmallVec::new(),
            })
        );
    }

    #[test]
    fn command_arguments() {
        assert_eq!(
            parse_directive(".command( a 1 ; b 2 ; c 3 ; ; ; )"),
            Some(Directive::Command {
                namespace: None,
                name: "command",
                indices: SmallVec::new(),
                suffix: None,
                arguments: smallvec::smallvec!["a 1", "b 2", "c 3"],
            })
        );
        // makes sure this isn't mistaken for indices
        assert_eq!(
            parse_directive(".command(0)"),
            Some(Directive::Command {
                namespace: None,
                name: "command",
                indices: SmallVec::new(),
                suffix: None,
                arguments: smallvec::smallvec!["0"],
            })
        );
        assert_eq!(
            parse_directive("f  .  command   ( a 1 ; b 2 ; c 3 )  "),
            Some(Directive::Command {
                namespace: Some("f"),
                name: "command",
                indices: SmallVec::new(),
                suffix: None,
                arguments: smallvec::smallvec!["a 1", "b 2", "c 3"],
            })
        );
        assert_eq!(
            parse_directive(".command a 1 ; b 2 ; c 3 "),
            Some(Directive::Command {
                namespace: None,
                name: "command",
                indices: SmallVec::new(),
                suffix: None,
                arguments: smallvec::smallvec!["a 1", "b 2", "c 3"],
            })
        );
        assert_eq!(
            parse_directive(".command a 1 ; b 2 ; c 3 ;;;;;;;; ;;;;"),
            Some(Directive::Command {
                namespace: None,
                name: "command",
                indices: SmallVec::new(),
                suffix: None,
                arguments: smallvec::smallvec!["a 1", "b 2", "c 3"],
            })
        );
    }

    #[test]
    fn command_indices() {
        assert_eq!(
            parse_directive(".command(-1;2;3;1) f"),
            Some(Directive::Command {
                namespace: None,
                name: "command",
                indices: smallvec::smallvec![-1, 2, 3, 1],
                suffix: None,
                arguments: smallvec::smallvec!["f"],
            })
        );
        assert_eq!(
            parse_directive(".command(-1;2;3;1;;  ;) f;; ;;"),
            Some(Directive::Command {
                namespace: None,
                name: "command",
                indices: smallvec::smallvec![-1, 2, 3, 1],
                suffix: None,
                arguments: smallvec::smallvec!["f"],
            })
        );
        // Parens around arguments forbidden after indices
        assert_eq!(parse_directive(".command(-1)(2)"), None);
    }

    #[test]
    fn command_suffix() {
        assert_eq!(
            parse_directive(".command(-1;2;3;1).h f"),
            Some(Directive::Command {
                namespace: None,
                name: "command",
                indices: smallvec::smallvec![-1, 2, 3, 1],
                suffix: Some("h"),
                arguments: smallvec::smallvec!["f"],
            })
        );
        assert_eq!(
            parse_directive(".command(-1;2;3;1).h(f)"),
            Some(Directive::Command {
                namespace: None,
                name: "command",
                indices: smallvec::smallvec![-1, 2, 3, 1],
                suffix: Some("h"),
                arguments: smallvec::smallvec!["f"],
            })
        );
        assert_eq!(
            parse_directive("namespace  . command (-1;2;3;1) . h (f;f2; 3;;)"),
            Some(Directive::Command {
                namespace: Some("namespace"),
                name: "command",
                indices: smallvec::smallvec![-1, 2, 3, 1],
                suffix: Some("h"),
                arguments: smallvec::smallvec!["f", "f2", "3"],
            })
        );
    }
}
