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
use smartstring::{LazyCompact, SmartString};
use std::{
    convert::{identity, TryFrom},
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

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // We never have more than one of these, so being large is fine.
pub enum Directive {
    TrackPosition(TrackPositionSmallVec),
    Command(Command),
    With(SmartString<LazyCompact>),
}

pub type TrackPositionSmallVec = SmallVec<[f32; 4]>;
pub type IndexSmallVec = SmallVec<[Option<i64>; 8]>;
pub type ArgumentSmallVec = SmallVec<[SmartString<LazyCompact>; 8]>;

pub fn parse_route(preprocessed: &str) -> impl Iterator<Item = Directive> + '_ {
    split_into_commands(preprocessed)
        .filter_map(apply_chr)
        .filter_map(remove_comments)
        .filter_map(parse_directive)
}

fn split_into_commands(input: &str) -> impl Iterator<Item = &str> {
    COMMAND_SPLIT_REGEX
        .split(input)
        .map(str::trim)
        .filter(|&s| !s.is_empty())
}

fn apply_chr(input: &str) -> Option<SmartString<LazyCompact>> {
    let mut output = SmartString::new();
    let mut last_capture = 0_usize;
    for capture in CHR_APPLY_REGEX.captures_iter(input) {
        let mat = capture.get(0).unwrap_or_else(|| unreachable!());
        output.push_str(&input[last_capture..mat.start()]);

        let number_str = capture.get(1).unwrap_or_else(|| unreachable!()).as_str();
        let number: u32 = number_str.parse().ok()?;
        output.push(char::try_from(number).ok()?);

        last_capture = mat.end();
    }
    output = SmartString::from(output.trim());
    Some(output)
}

fn remove_comments(input: SmartString<LazyCompact>) -> Option<SmartString<LazyCompact>> {
    if input.chars().next()? == ';' {
        None
    } else {
        Some(input)
    }
}

#[allow(clippy::needless_pass_by_value)] // this is only used in a iterator map call, so we need to take exactly what we get
fn parse_directive(command: SmartString<LazyCompact>) -> Option<Directive> {
    alt((
        parse_with,
        parse_track_position,
        parse_command_indices_args,
        parse_command_args,
        parse_command,
    ))(&command)
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
        parse_argument_list,
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
                (command, (None, name))
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
            parse_argument_list1,
        ))(command)?
    } else {
        parse_argument_list1(command)?
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

fn parse_argument(command: &str) -> IResult<&str, Option<&str>> {
    opt(w(is_not("(;)")))(command).map_output(|opt| opt.map(str::trim))
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
        assert_eq!(parse_directive(ss!("With Blob")), Some(Directive::With(ss!("Blob"))));
        assert_eq!(
            parse_directive(ss!("With    BlobH ")),
            Some(Directive::With(ss!("BlobH")))
        );
        assert_eq!(parse_directive(ss!("With")), None);
    }

    macro_rules! smallvec_opt {
        ($($value:expr),*) => {smallvec::smallvec![$(Some($value)),*]};
    }

    #[test]
    fn track_position() {
        assert_eq!(
            parse_directive(ss!("1000")),
            Some(Directive::TrackPosition(smallvec::smallvec![1000.0]))
        );
        assert_eq!(
            parse_directive(ss!("1000 ;;; ; ; ;; ;")),
            Some(Directive::TrackPosition(smallvec::smallvec![1000.0]))
        );
        assert_eq!(
            parse_directive(ss!("1000;2000")),
            Some(Directive::TrackPosition(smallvec::smallvec![1000.0, 2000.0]))
        );
        assert_eq!(
            parse_directive(ss!("1000  ; 2000")),
            Some(Directive::TrackPosition(smallvec::smallvec![1000.0, 2000.0]))
        );
        assert_eq!(
            parse_directive(ss!("1000.42;2000.84")),
            Some(Directive::TrackPosition(smallvec::smallvec![1000.42, 2000.84]))
        );
        assert_eq!(parse_directive(ss!("")), None);
        assert_eq!(parse_directive(ss!(";")), None);
    }

    #[test]
    fn command() {
        assert_eq!(
            parse_directive(ss!(".command")),
            Some(Directive::Command(Command {
                name: ss!("command"),
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(ss!("  .  command  ")),
            Some(Directive::Command(Command {
                name: ss!("command"),
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(ss!("namespace.command")),
            Some(Directive::Command(Command {
                namespace: Some(ss!("namespace")),
                name: ss!("command"),
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(ss!("  namespace .  command  ")),
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
            parse_directive(ss!(".command( a 1 ; b 2 ; c 3 ; ; ; )")),
            Some(Directive::Command(Command {
                name: ss!("command"),
                arguments: smallvec::smallvec![ss!("a 1"), ss!("b 2"), ss!("c 3"), ss!(""), ss!(""), ss!("")],
                ..default_command()
            }))
        );
        // makes sure this isn't mistaken for indices
        assert_eq!(
            parse_directive(ss!(".command(0)")),
            Some(Directive::Command(Command {
                name: ss!("command"),
                arguments: smallvec::smallvec![ss!("0")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(ss!("f  .  command   ( a 1 ; b 2 ; c 3 )  ")),
            Some(Directive::Command(Command {
                namespace: Some(ss!("f")),
                name: ss!("command"),
                arguments: smallvec::smallvec![ss!("a 1"), ss!("b 2"), ss!("c 3")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(ss!(".command a 1 ; b 2 ; c 3 ")),
            Some(Directive::Command(Command {
                name: ss!("command"),
                arguments: smallvec::smallvec![ss!("a 1"), ss!("b 2"), ss!("c 3")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(ss!(".command a 1 ; b 2 ; c 3 ; ; ; ")),
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
            parse_directive(ss!(".command(-1;2;3;1) f")),
            Some(Directive::Command(Command {
                name: ss!("command"),
                indices: smallvec_opt![-1, 2, 3, 1],
                arguments: smallvec::smallvec![ss!("f")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(ss!(".command(-1;2;3;1;;) f;; ;;")),
            Some(Directive::Command(Command {
                name: ss!("command"),
                indices: smallvec::smallvec![Some(-1), Some(2), Some(3), Some(1), None, None],
                arguments: smallvec::smallvec![ss!("f"), ss!(""), ss!(""), ss!(""), ss!("")],
                ..default_command()
            }))
        );
        // Parens around arguments forbidden after indices
        assert_eq!(parse_directive(ss!(".command(-1)(2)")), None);
    }

    #[test]
    fn command_suffix() {
        assert_eq!(
            parse_directive(ss!(".command(-1;2;3;1).h f")),
            Some(Directive::Command(Command {
                name: ss!("command"),
                indices: smallvec_opt![-1, 2, 3, 1],
                suffix: Some(ss!("h")),
                arguments: smallvec::smallvec![ss!("f")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(ss!(".command(-1;2;3;1).h(f)")),
            Some(Directive::Command(Command {
                name: ss!("command"),
                indices: smallvec_opt![-1, 2, 3, 1],
                suffix: Some(ss!("h")),
                arguments: smallvec::smallvec![ss!("f")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(ss!(".command(-1;2;3;1).h.j f")),
            Some(Directive::Command(Command {
                name: ss!("command"),
                indices: smallvec_opt![-1, 2, 3, 1],
                suffix: Some(ss!("h")),
                arguments: smallvec::smallvec![ss!("f")],
                ..default_command()
            }))
        );
        assert_eq!(
            parse_directive(ss!("namespace  . command (-1;2;3;1) . h (f;f2; 3;;)")),
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
            parse_directive(ss!("signal(2).Load H; K")),
            Some(Directive::Command(Command {
                name: ss!("signal"),
                indices: smallvec_opt![2],
                suffix: Some(ss!("Load")),
                arguments: smallvec::smallvec![ss!("H"), ss!("K")],
                ..default_command()
            }))
        );
    }
}
