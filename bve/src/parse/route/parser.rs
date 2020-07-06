use bve_common::nom::{separated_list_small, w, MapOutput};
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag_no_case, take_until, take_while},
    combinator::{map_res, opt},
    sequence::{delimited, preceded, separated_pair},
    IResult,
};
use once_cell::sync::Lazy;
use regex::Regex;
use smallvec::SmallVec;

static COMMAND_SPLIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[,\r\n]").expect("invalid regex"));

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
    COMMAND_SPLIT_REGEX.split(input).map(str::trim)
}

fn parse_directive(command: &str) -> Option<Directive<'_>> {
    alt((
        parse_with,
        parse_track_position,
        parse_command_indices_args,
        parse_command_args,
        parse_command,
    ))(command)
    .map(|(_, directive)| directive)
    .ok()
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
        delimited(w(tag_no_case("(")), parse_argument_list, w(tag_no_case(")"))),
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
    let (command, indices) = delimited(
        w(tag_no_case("(")),
        separated_list_small(w(tag_no_case(";")), parse_integer_number),
        w(tag_no_case(")")),
    )(command)?;
    let (command, suffix) = preceded(w(tag_no_case(".")), opt(parse_identifier))(command)?;
    let (command, arguments) = if suffix.is_some() {
        alt((
            delimited(w(tag_no_case("(")), parse_argument_list, w(tag_no_case(")"))),
            parse_argument_list,
        ))(command)?
    } else {
        parse_argument_list(command)?
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
    separated_list_small(w(tag_no_case(";")), parse_floating_number)(command)
        .map_output(|list| Directive::TrackPosition(list))
}

fn parse_floating_number(command: &str) -> IResult<&str, f32> {
    map_res(w(is_a("0123456789.-E")), |s: &str| s.parse())(command)
}

fn parse_integer_number(command: &str) -> IResult<&str, i64> {
    map_res(w(is_a("0123456789-")), |s: &str| s.parse())(command)
}

fn parse_identifier(command: &str) -> IResult<&str, &str> {
    w(take_while(char::is_alphabetic))(command)
}

fn parse_argument_list(command: &str) -> IResult<&str, ArgumentSmallVec<'_>> {
    separated_list_small(w(tag_no_case(";")), parse_argument)(command)
}

fn parse_argument(command: &str) -> IResult<&str, &str> {
    w(take_until(";"))(command).map_output(str::trim)
}
