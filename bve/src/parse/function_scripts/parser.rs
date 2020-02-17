use crate::parse::function_scripts::Instruction;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{char as char_f, one_of};
use nom::combinator::{map_res, opt};
use nom::multi::many0;
use nom::sequence::{delimited, tuple};
use nom::{AsChar, IResult, InputTakeAtPosition};
use std::num::ParseFloatError;

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

fn binary_left<F1, F2, F3, I, O1, O2, O3>(
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

fn binary_right<F1, F2, F3, I, O1, O2, O3>(
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

enum OneOrManyInstructions {
    One(Instruction),
    Many(Vec<Instruction>),
}

impl From<OneOrManyInstructions> for Vec<Instruction> {
    fn from(rhs: OneOrManyInstructions) -> Self {
        match rhs {
            OneOrManyInstructions::One(instruction) => vec![instruction],
            OneOrManyInstructions::Many(instructions) => instructions,
        }
    }
}

pub fn parse_function_script(input: &str) -> IResult<&str, Vec<Instruction>> {
    expression(input)
}

pub fn expression(input: &str) -> IResult<&str, Vec<Instruction>> {
    logical_or(input)
}

fn logical_or(input: &str) -> IResult<&str, Vec<Instruction>> {
    binary_right(logical_xor, char_f('|'), logical_or)(input).map(|(input, (mut left, right_opt))| {
        if let Some((_, right)) = right_opt {
            left.extend(right.into_iter());
            left.push(Instruction::LogicalOr);
        }
        (input, left)
    })
}

fn logical_xor(input: &str) -> IResult<&str, Vec<Instruction>> {
    binary_right(logical_and, char_f('^'), logical_xor)(input).map(|(input, (mut left, right_opt))| {
        if let Some((_, right)) = right_opt {
            left.extend(right.into_iter());
            left.push(Instruction::LogicalXor);
        }
        (input, left)
    })
}

fn logical_and(input: &str) -> IResult<&str, Vec<Instruction>> {
    binary_right(logical_not, char_f('&'), logical_and)(input).map(|(input, (mut left, right_opt))| {
        if let Some((_, right)) = right_opt {
            left.extend(right.into_iter());
            left.push(Instruction::LogicalAnd);
        }
        (input, left)
    })
}

fn logical_not(input: &str) -> IResult<&str, Vec<Instruction>> {
    unary(char_f('!'), equal_expr)(input).map(|(input, (operator, mut child))| {
        if let Some(_) = operator {
            child.push(Instruction::UnaryLogicalNot);
        }
        (input, child)
    })
}

fn equal_symbol(input: &str) -> IResult<&str, &str> {
    w(alt((tag("=="), tag("!="), tag("<="), tag(">="), tag(">"), tag("<"))))(input)
}

fn equal_expr(input: &str) -> IResult<&str, Vec<Instruction>> {
    binary_left(plus_expr, equal_symbol, plus_expr)(input).map(|(input, (mut left, right_vec))| {
        for (operator, right) in right_vec {
            left.extend(right.into_iter());
            left.push(match operator {
                "==" => Instruction::Equal,
                "!=" => Instruction::NotEqual,
                "<" => Instruction::Less,
                "<=" => Instruction::LessEqual,
                ">" => Instruction::Greater,
                ">=" => Instruction::GreaterEqual,
                _ => unreachable!(),
            });
        }
        (input, left)
    })
}

fn plus_symbol(input: &str) -> IResult<&str, char> {
    w(alt((char_f('+'), char_f('-'))))(input)
}

fn plus_expr(input: &str) -> IResult<&str, Vec<Instruction>> {
    // This is left associative while the rest of everything is right associative
    // Idk man
    binary_left(times_expr, plus_symbol, times_expr)(input).map(|(input, (mut left, right_vec))| {
        for (operator, right) in right_vec {
            left.extend(right.into_iter());
            left.push(if operator == '+' {
                Instruction::Addition
            } else {
                Instruction::Subtraction
            });
        }
        (input, left)
    })
}

fn times_expr(input: &str) -> IResult<&str, Vec<Instruction>> {
    binary_right(divide_expr, char_f('*'), times_expr)(input).map(|(input, (mut left, right_opt))| {
        if let Some((_, right)) = right_opt {
            left.extend(right.into_iter());
            left.push(Instruction::Multiplication);
        }
        (input, left)
    })
}

fn divide_expr(input: &str) -> IResult<&str, Vec<Instruction>> {
    binary_right(unary_negative_expr, char_f('/'), divide_expr)(input).map(|(input, (mut left, right_opt))| {
        if let Some((_, right)) = right_opt {
            left.extend(right.into_iter());
            left.push(Instruction::Division);
        }
        (input, left)
    })
}

fn unary_negative_expr(input: &str) -> IResult<&str, Vec<Instruction>> {
    unary(char_f('-'), function_expr)(input).map(|(input, (operator, mut child))| {
        if let Some(_) = operator {
            child.push(Instruction::UnaryNegative);
        }
        (input, child)
    })
}

fn function_expr(input: &str) -> IResult<&str, Vec<Instruction>> {
    alt((function_call, term))(input)
}

fn function_call(input: &str) -> IResult<&str, Vec<Instruction>> {
    tuple((
        name,
        w(char_f('[')),
        expression,
        many0(tuple((w(char_f(',')), expression))),
        w(char_f(']')),
    ))(input)
    .map(
        |(input, (name, _, arg1, argn, _)): (&str, (String, _, Vec<Instruction>, Vec<(char, Vec<Instruction>)>, _))| {
            let mut instructions = Vec::new();
            instructions.extend(arg1);
            let arg_count = argn.len() + 1;
            argn.into_iter()
                .for_each(|(_, arg)| instructions.extend(arg.into_iter()));
            instructions.push(Instruction::FunctionCall { name, arg_count });
            (input, instructions)
        },
    )
}

fn term(input: &str) -> IResult<&str, Vec<Instruction>> {
    alt((parens_expr, variable_name, number))(input)
        .map(|(input, child_instructions)| (input, Vec::<Instruction>::from(child_instructions)))
}

fn variable_name(input: &str) -> IResult<&str, OneOrManyInstructions> {
    name(input).map(|(input, name)| (input, OneOrManyInstructions::One(Instruction::Variable { name })))
}

fn parens_expr(input: &str) -> IResult<&str, OneOrManyInstructions> {
    tuple((w(char_f('(')), expression, w(char_f(')'))))(input)
        .map(|(input, (_, expr, _))| (input, OneOrManyInstructions::Many(expr)))
}

fn number(input: &str) -> IResult<&str, OneOrManyInstructions> {
    map_res(
        w(take_while(char::is_dec_digit)),
        |digits: &str| -> Result<OneOrManyInstructions, ParseFloatError> {
            let value: f64 = digits.parse()?;
            Ok(OneOrManyInstructions::One(Instruction::Number { value }))
        },
    )(input)
}

fn name(input: &str) -> IResult<&str, String> {
    w(tuple((letter, many0(alt((letter, digit))))))(input).map(|(input, (first_char, remaining))| {
        let mut name = String::new();
        name.push(first_char);
        name.extend(remaining);
        (input, name)
    })
}

// TODO: Optimize these routines
fn letter(input: &str) -> IResult<&str, char> {
    one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")(input)
}

fn digit(input: &str) -> IResult<&str, char> {
    one_of("0123456789.")(input)
}

#[cfg(test)]
mod test {
    use crate::parse::function_scripts::parser::parse_function_script;
    use crate::parse::function_scripts::Instruction;

    macro_rules! function_script_assert {
        ($input:expr, $($result:expr),* ,) => {
            let input = $input;
            let (remaining, output) = parse_function_script(input.as_ref()).unwrap();
            assert_eq!(remaining, "");

            itertools::assert_equal(
                output.into_iter(),
                vec![
                    $($result),*
                ]
                .into_iter(),
            )
        };
    }

    #[test]
    fn addition() {
        // Left associative
        function_script_assert!(
            "1 + 2 + 3",
            Instruction::Number { value: 1.0 },
            Instruction::Number { value: 2.0 },
            Instruction::Addition,
            Instruction::Number { value: 3.0 },
            Instruction::Addition,
        );
    }

    #[test]
    fn subtraction() {
        // Left associative
        function_script_assert!(
            "1 - 2 - 3",
            Instruction::Number { value: 1.0 },
            Instruction::Number { value: 2.0 },
            Instruction::Subtraction,
            Instruction::Number { value: 3.0 },
            Instruction::Subtraction,
        );
    }

    #[test]
    fn multiplication() {
        function_script_assert!(
            "1 * 2 * 3",
            Instruction::Number { value: 1.0 },
            Instruction::Number { value: 2.0 },
            Instruction::Number { value: 3.0 },
            Instruction::Multiplication,
            Instruction::Multiplication,
        );
    }

    #[test]
    fn division() {
        function_script_assert!(
            "1 / 2 / 3",
            Instruction::Number { value: 1.0 },
            Instruction::Number { value: 2.0 },
            Instruction::Number { value: 3.0 },
            Instruction::Division,
            Instruction::Division,
        );
    }

    #[test]
    fn equality() {
        let operators = [
            ("<", Instruction::Less),
            ("<=", Instruction::LessEqual),
            ("==", Instruction::Equal),
            ("!=", Instruction::NotEqual),
            (">", Instruction::Greater),
            (">=", Instruction::GreaterEqual),
        ];
        for (string, instruction) in operators.iter() {
            function_script_assert!(
                format!("1 {0} 2 {0} 3", string),
                Instruction::Number { value: 1.0 },
                Instruction::Number { value: 2.0 },
                instruction.clone(),
                Instruction::Number { value: 3.0 },
                instruction.clone(),
            );
        }
    }

    #[test]
    fn parens() {
        function_script_assert!(
            "1 + (2 + 3)",
            Instruction::Number { value: 1.0 },
            Instruction::Number { value: 2.0 },
            Instruction::Number { value: 3.0 },
            Instruction::Addition,
            Instruction::Addition,
        );
    }

    #[test]
    fn order_of_operations() {
        function_script_assert!(
            "1 + 2 - 3 * 4 / 5",
            Instruction::Number { value: 1.0 },
            Instruction::Number { value: 2.0 },
            Instruction::Addition,
            Instruction::Number { value: 3.0 },
            Instruction::Number { value: 4.0 },
            Instruction::Number { value: 5.0 },
            Instruction::Division,
            Instruction::Multiplication,
            Instruction::Subtraction,
        );
        function_script_assert!(
            "1 / 2 * 3 - 4 + 5",
            Instruction::Number { value: 1.0 },
            Instruction::Number { value: 2.0 },
            Instruction::Division,
            Instruction::Number { value: 3.0 },
            Instruction::Multiplication,
            Instruction::Number { value: 4.0 },
            Instruction::Subtraction,
            Instruction::Number { value: 5.0 },
            Instruction::Addition,
        );
    }
}
