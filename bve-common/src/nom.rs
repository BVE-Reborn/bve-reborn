use nom::{
    bytes::complete::take_while,
    combinator::opt,
    multi::many0,
    sequence::{delimited, tuple},
    IResult, InputTakeAtPosition,
};

/// Takes a parser and wraps it so it delimited with whitespace
pub fn w<F, I, O>(func: F) -> impl Fn(I) -> IResult<I, O>
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

pub fn binary_left<F1, F2, F3, I, O1, O2, O3>(
    left: F1,
    middle: F2,
    right: F3,
) -> impl FnOnce(I) -> IResult<I, (O1, Vec<(O2, O3)>)>
where
    I: InputTakeAtPosition<Item = char> + Clone + PartialEq,
    F1: Fn(I) -> IResult<I, O1>,
    F2: Fn(I) -> IResult<I, O2>,
    F3: Fn(I) -> IResult<I, O3>,
{
    move |input| tuple((left, many0(tuple((middle, right)))))(input)
}

pub fn binary_right<F1, F2, F3, I, O1, O2, O3>(
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

pub fn unary<F1, F2, I, O1, O2>(left: F1, right: F2) -> impl FnOnce(I) -> IResult<I, (Option<O1>, O2)>
where
    I: InputTakeAtPosition<Item = char> + Clone,
    F1: Fn(I) -> IResult<I, O1>,
    F2: Fn(I) -> IResult<I, O2>,
{
    move |input| tuple((opt(w(left)), right))(input)
}
