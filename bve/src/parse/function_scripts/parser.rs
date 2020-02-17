use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{char as char_f, one_of};
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::{delimited, tuple};
use nom::{AsChar, IResult, InputTakeAtPosition};

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

fn binary<F1, F2, F3, I, O1, O2, O3>(
    left: F1,
    middle: F2,
    right: F3,
) -> impl FnOnce(I) -> IResult<I, (O1, Option<(O2, O3)>)>
where
    I: InputTakeAtPosition<Item = char> + Clone,
    F1: Fn(I) -> IResult<I, O1>,
    F2: Fn(I) -> IResult<I, O2>,
    F3: Fn(I) -> IResult<I, O3>,
{
    move |input| tuple((left, opt(tuple((middle, right)))))(input)
}

fn unary<F1, F2, I, O1, O2>(left: F1, right: F2) -> impl FnOnce(I) -> IResult<I, (Option<O1>, O2)>
where
    I: InputTakeAtPosition<Item = char> + Clone,
    F1: Fn(I) -> IResult<I, O1>,
    F2: Fn(I) -> IResult<I, O2>,
{
    move |input| tuple((opt(w(left)), right))(input)
}

pub fn parse_function_script(input: &str) -> IResult<&str, ()> {
    expression(input)
}

pub fn expression(input: &str) -> IResult<&str, ()> {
    logical_or(input)
}

fn logical_or(input: &str) -> IResult<&str, ()> {
    binary(logical_xor, char_f('|'), logical_or)(input).map(|(input, _)| (input, ()))
}

fn logical_xor(input: &str) -> IResult<&str, ()> {
    binary(logical_and, char_f('^'), logical_xor)(input).map(|(input, _)| (input, ()))
}

fn logical_and(input: &str) -> IResult<&str, ()> {
    binary(logical_not, char_f('^'), logical_xor)(input).map(|(input, _)| (input, ()))
}

fn logical_not(input: &str) -> IResult<&str, ()> {
    unary(char_f('!'), equal_expr)(input).map(|(input, _)| (input, ()))
}

fn equal_symbol(input: &str) -> IResult<&str, &str> {
    w(alt((tag("=="), tag("!="), tag(">"), tag("<"), tag("<="), tag(">="))))(input)
}

fn equal_expr(input: &str) -> IResult<&str, ()> {
    binary(plus_expr, equal_symbol, equal_expr)(input).map(|(input, _)| (input, ()))
}

fn plus_symbol(input: &str) -> IResult<&str, char> {
    w(alt((char_f('+'), char_f('-'))))(input)
}

fn plus_expr(input: &str) -> IResult<&str, ()> {
    binary(times_expr, plus_symbol, plus_expr)(input).map(|(input, _)| (input, ()))
}

fn times_expr(input: &str) -> IResult<&str, ()> {
    binary(divide_expr, char_f('*'), times_expr)(input).map(|(input, _)| (input, ()))
}

fn divide_expr(input: &str) -> IResult<&str, ()> {
    binary(unary_minus_expr, char_f('*'), times_expr)(input).map(|(input, _)| (input, ()))
}

fn unary_minus_expr(input: &str) -> IResult<&str, ()> {
    unary(char_f('-'), function_expr)(input).map(|(input, _)| (input, ()))
}

fn function_expr(input: &str) -> IResult<&str, ()> {
    alt((function_call, term))(input).map(|(input, _)| (input, ()))
}

fn function_call(input: &str) -> IResult<&str, ()> {
    tuple((
        name,
        w(char_f('[')),
        expression,
        many0(tuple((w(char_f(',')), expression))),
        w(char_f(']')),
    ))(input)
    .map(|(input, _)| (input, ()))
}

fn term(input: &str) -> IResult<&str, ()> {
    alt((parens_expr, name, number))(input).map(|(input, _)| (input, ()))
}

fn parens_expr(input: &str) -> IResult<&str, ()> {
    tuple((w(char_f('(')), expression, w(char_f(')'))))(input).map(|(input, _)| (input, ()))
}

fn number(input: &str) -> IResult<&str, ()> {
    w(take_while(char::is_dec_digit))(input).map(|(input, _)| (input, ()))
}

fn name(input: &str) -> IResult<&str, ()> {
    w(tuple((letter, many0(alt((letter, digit))))))(input).map(|(input, _)| (input, ()))
}

fn letter(input: &str) -> IResult<&str, ()> {
    one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")(input).map(|(input, _)| (input, ()))
}

fn digit(input: &str) -> IResult<&str, ()> {
    one_of("0123456789")(input).map(|(input, _)| (input, ()))
}

#[cfg(test)]
mod test {
    use crate::parse::function_scripts::parser::parse_function_script;

    #[test]
    fn addition() {
        let (input_left, _) = parse_function_script("1 + 2 + 3").unwrap();
        assert!(input_left.is_empty());
    }
}
