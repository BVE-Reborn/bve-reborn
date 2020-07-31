use nom::{
    bytes::complete::take_while,
    combinator::opt,
    error::{ErrorKind, ParseError},
    sequence::{delimited, tuple},
    IResult, InputTakeAtPosition,
};
use smallvec::SmallVec;

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

pub fn binary_left<Array, F1, F2, F3, I, O1, O2, O3>(
    left: F1,
    middle: F2,
    right: F3,
) -> impl FnOnce(I) -> IResult<I, (O1, SmallVec<Array>)>
where
    Array: smallvec::Array<Item = (O2, O3)>,
    I: InputTakeAtPosition<Item = char> + Clone + PartialEq,
    F1: Fn(I) -> IResult<I, O1>,
    F2: Fn(I) -> IResult<I, O2>,
    F3: Fn(I) -> IResult<I, O3>,
{
    move |input| tuple((left, many0_small(tuple((middle, right)))))(input)
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

// Copied from nom, converted to smallvec
pub fn separated_list_small<Array, I, O, O2, E, F, G>(sep: G, f: F) -> impl Fn(I) -> IResult<I, SmallVec<Array>, E>
where
    Array: smallvec::Array<Item = O>,
    I: Clone + PartialEq,
    F: Fn(I) -> IResult<I, O, E>,
    G: Fn(I) -> IResult<I, O2, E>,
    E: ParseError<I>,
{
    move |mut i: I| {
        let mut res = SmallVec::new();

        match f(i.clone()) {
            Err(nom::Err::Error(_)) => return Ok((i, res)),
            Err(e) => return Err(e),
            Ok((i1, o)) => {
                if i1 == i {
                    return Err(nom::Err::Error(E::from_error_kind(i1, ErrorKind::SeparatedList)));
                }

                res.push(o);
                i = i1;
            }
        }

        loop {
            match sep(i.clone()) {
                Err(nom::Err::Error(_)) => return Ok((i, res)),
                Err(e) => return Err(e),
                Ok((i1, _)) => {
                    if i1 == i {
                        return Err(nom::Err::Error(E::from_error_kind(i1, ErrorKind::SeparatedList)));
                    }

                    match f(i1.clone()) {
                        Err(nom::Err::Error(_)) => return Ok((i, res)),
                        Err(e) => return Err(e),
                        Ok((i2, o)) => {
                            if i2 == i {
                                return Err(nom::Err::Error(E::from_error_kind(i2, ErrorKind::SeparatedList)));
                            }

                            res.push(o);
                            i = i2;
                        }
                    }
                }
            }
        }
    }
}

pub fn many0_small<Array, I, O, E, F>(f: F) -> impl Fn(I) -> IResult<I, SmallVec<Array>, E>
where
    Array: smallvec::Array<Item = O>,
    I: Clone + PartialEq,
    F: Fn(I) -> IResult<I, O, E>,
    E: ParseError<I>,
{
    move |mut i: I| {
        let mut acc = SmallVec::new();
        loop {
            match f(i.clone()) {
                Err(nom::Err::Error(_)) => return Ok((i, acc)),
                Err(e) => return Err(e),
                Ok((i1, o)) => {
                    if i1 == i {
                        return Err(nom::Err::Error(E::from_error_kind(i, ErrorKind::Many0)));
                    }

                    i = i1;
                    acc.push(o);
                }
            }
        }
    }
}

pub trait MapOutput<I, O> {
    #[allow(clippy::missing_errors_doc)]
    fn map_output<F, O2>(self, func: F) -> IResult<I, O2>
    where
        F: FnOnce(O) -> O2;
}

impl<I, O> MapOutput<I, O> for IResult<I, O> {
    fn map_output<F, O2>(self, func: F) -> IResult<I, O2>
    where
        F: FnOnce(O) -> O2,
    {
        self.map(move |(i, o)| (i, func(o)))
    }
}
