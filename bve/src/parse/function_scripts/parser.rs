use nom::branch::alt;
use nom::bytes::complete::take_while;
use nom::character::complete::char as char_f;
use nom::combinator::opt;
use nom::sequence::delimited;
use nom::{IResult, InputTakeAtPosition};

/// Takes a parser and wraps it so it delimited with whitespace
fn w<F, I, O>(func: F) -> impl Fn(I) -> IResult<I, O>
where
    I: InputTakeAtPosition<Item = char>,
    F: Fn(I) -> IResult<I, O>,
{
    move |input| {
        delimited(
            take_while(char::is_whitespace), //
            &func,
            take_while(char::is_whitespace),
        )(input)
    }
}

pub fn parse_function_script(input: &str) -> IResult<&str, ()> {
    logical_or(input)
}

fn logical_or(input: &str) -> IResult<&str, ()> {
    delimited(logical_xor, w(char_f('|')), logical_or)(input).map(|(input, _)| (input, ()))
}

fn logical_xor(input: &str) -> IResult<&str, ()> {
    delimited(logical_and, w(char_f('^')), logical_xor)(input).map(|(input, _)| (input, ()))
}

fn logical_and(input: &str) -> IResult<&str, ()> {
    delimited(logical_not, w(char_f('^')), logical_xor)(input).map(|(input, _)| (input, ()))
}

fn logical_not(input: &str) -> IResult<&str, ()> {
    alt((opt(w(char_f('!'))), equal_expr))(input).map(|(input, _)| (input, ()))
}

fn equal_expr(input: &str) -> IResult<&str, ()> {}

fn equal_symbol(input: &str) -> IResult<&str, ()> {}
