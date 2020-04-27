use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while},
    combinator::opt,
    multi::separated_list,
    sequence::{delimited, preceded, tuple},
    IResult, InputTakeAtPosition,
};

#[derive(Debug, Copy, Clone)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Debug, Clone)]
pub struct ShaderCombination<'a> {
    pub name: &'a str,
    pub ty: ShaderType,
    pub defines: Vec<SingleDefine<'a>>,
}

pub fn parse_shader_compile_file(input: &str) -> Result<impl Iterator<Item = ShaderCombination<'_>>, &str> {
    separated_list(alt((tag("\r\n"), tag("\n"))), parse_shader_directive)(input)
        .map_err(|_| "parsing error")
        .and_then(|(str, v)| {
            if !str.is_empty() {
                return Err(str);
            }
            Ok(v.into_iter().flatten())
        })
}

/// Takes a parser and wraps it so it delimited with whitespace
fn w<F, I, O>(func: F) -> impl Fn(I) -> IResult<I, O>
where
    I: InputTakeAtPosition<Item = char>,
    F: Fn(I) -> IResult<I, O>,
{
    move |input| {
        delimited(
            take_while(|c: char| c.is_whitespace() && c != '\n' && c != '\r'), //
            &func,
            take_while(|c: char| c.is_whitespace() && c != '\n' && c != '\r'),
        )(input)
    }
}

fn parse_shader_directive(input: &str) -> IResult<&str, Vec<ShaderCombination<'_>>> {
    tuple((
        w(parse_word),
        w(tag("-")),
        w(parse_shader_type),
        w(tag(":")),
        parse_shader_permutations,
    ))(input)
    .map(|(s, (name, _, ty, _, permutations))| {
        (s, {
            let vec = permutations
                .map(move |perm| ShaderCombination {
                    name,
                    ty,
                    defines: perm,
                })
                .collect_vec();

            if vec.is_empty() {
                vec![ShaderCombination {
                    name,
                    ty,
                    defines: Vec::new(),
                }]
            } else {
                vec
            }
        })
    })
}

fn parse_word(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_alphanumeric() || c == '_')(input)
}

fn parse_shader_type(input: &str) -> IResult<&str, ShaderType> {
    alt((
        tag_no_case("vertex"),
        tag_no_case("vert"),
        tag_no_case("fragment"),
        tag_no_case("frag"),
        tag_no_case("compute"),
        tag_no_case("comp"),
    ))(input)
    .map(|(s, value)| {
        (s, match value.to_lowercase().as_str() {
            "vert" | "vertex" => ShaderType::Vertex,
            "frag" | "fragment" => ShaderType::Fragment,
            "comp" | "compute" => ShaderType::Compute,
            _ => unreachable!(),
        })
    })
}

#[derive(Debug, Clone)]
pub enum SingleDefine<'a> {
    Defined(&'a str, &'a str),
    Undefined(&'a str),
}

fn parse_shader_permutations(input: &str) -> IResult<&str, impl Iterator<Item = Vec<SingleDefine<'_>>>> {
    separated_list(w(tag_no_case("and")), parse_shader_define_set)(input)
        .map(|(s, vec)| (s, vec.into_iter().multi_cartesian_product()))
}

fn parse_shader_define_set(input: &str) -> IResult<&str, Vec<SingleDefine<'_>>> {
    separated_list(w(tag_no_case("or")), parse_single_optional_define)(input)
}

fn parse_single_optional_define(input: &str) -> IResult<&str, SingleDefine<'_>> {
    tuple((opt(w(tag_no_case("not"))), parse_shader_define))(input).map(|(s, (opt_not, (key, value)))| {
        (
            s,
            if opt_not.is_some() {
                SingleDefine::Undefined(key)
            } else {
                SingleDefine::Defined(key, value)
            },
        )
    })
}

fn parse_shader_define(input: &str) -> IResult<&str, (&str, &str)> {
    tuple((parse_word, opt(preceded(w(tag("=")), parse_word))))(input)
        .map(|(s, (key, opt_value)): (&str, (&str, Option<&str>))| (s, (key, opt_value.unwrap_or_else(|| "1"))))
}
