use crate::parse::function_scripts::{Instruction, ParsedFunctionScript};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{char as char_f, one_of},
    combinator::{map_res, opt},
    multi::many0,
    sequence::{delimited, tuple},
    AsChar, IResult, InputTakeAtPosition,
};
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

/// Parses the function script string into function script IR.
///
/// # Errors
///
/// - Errors when function script grammar is violated and was unable to be parsed. There may be leftover input, this is
///   likely a bug, but not enough to halt execution.
pub fn parse_function_script(input: &str) -> IResult<&str, ParsedFunctionScript> {
    expression(input).map(|(input, output)| (input, output.into()))
}

fn expression(input: &str) -> IResult<&str, Vec<Instruction>> {
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
        if operator.is_some() {
            child.push(Instruction::UnaryLogicalNot);
        }
        (input, child)
    })
}

fn equal_symbol(input: &str) -> IResult<&str, &str> {
    w(alt((tag("=="), tag("!="), tag("<="), tag(">="), tag(">"), tag("<"))))(input)
}

fn equal_expr(input: &str) -> IResult<&str, Vec<Instruction>> {
    // This is left associative while the rest of everything is right associative
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
        if operator.is_some() {
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
    .map(|(input, (name, _, arg1, argn, _))| {
        let mut instructions = Vec::new();
        instructions.extend(arg1);
        let arg_count = argn.len() + 1;
        argn.into_iter()
            .for_each(|(_, arg)| instructions.extend(arg.into_iter()));
        instructions.push(Instruction::FunctionCall { name, arg_count });
        (input, instructions)
    })
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
        w(take_while(|c: char| {
            c.is_dec_digit() || c == '.' || c == 'e' || c == 'E'
        })),
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
    use crate::parse::function_scripts::{parser::parse_function_script, Instruction};

    macro_rules! function_script_assert {
        ($input:expr, $($result:expr),* ,) => {{
            let input = $input;
            let (remaining, output) = parse_function_script(input.as_ref()).unwrap();
            assert_eq!(remaining, "");

            itertools::assert_equal(
                output.instructions.into_iter(),
                vec![
                    $($result),*
                ]
                .into_iter(),
            )
        }};
    }

    #[test]
    fn left_associative_operators() {
        let operators = [
            ("+", Instruction::Addition),
            ("-", Instruction::Subtraction),
            ("<", Instruction::Less),
            ("<=", Instruction::LessEqual),
            ("==", Instruction::Equal),
            ("!=", Instruction::NotEqual),
            (">", Instruction::Greater),
            (">=", Instruction::GreaterEqual),
        ];
        // Left associative
        for (string, instruction) in &operators {
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
    fn right_associative_operators() {
        let operators = [
            ("/", Instruction::Division),
            ("*", Instruction::Multiplication),
            ("&", Instruction::LogicalAnd),
            ("|", Instruction::LogicalOr),
            ("^", Instruction::LogicalXor),
        ];
        // Right associative
        for (string, instruction) in &operators {
            function_script_assert!(
                format!("1 {0} 2 {0} 3", string),
                Instruction::Number { value: 1.0 },
                Instruction::Number { value: 2.0 },
                Instruction::Number { value: 3.0 },
                instruction.clone(),
                instruction.clone(),
            );
        }
    }

    #[test]
    fn unary_operators() {
        let operators = [("!", Instruction::UnaryLogicalNot), ("-", Instruction::UnaryNegative)];
        for (string, instruction) in &operators {
            function_script_assert!(
                format!("{0} 1", string),
                Instruction::Number { value: 1.0 },
                instruction.clone(),
            );
            function_script_assert!(
                format!("{0}(1)", string),
                Instruction::Number { value: 1.0 },
                instruction.clone(),
            );
            function_script_assert!(
                format!("{0}(1 + 2)", string),
                Instruction::Number { value: 1.0 },
                Instruction::Number { value: 2.0 },
                Instruction::Addition,
                instruction.clone(),
            );
        }
    }

    #[test]
    fn function_call() {
        function_script_assert!(
            "func[1]",
            Instruction::Number { value: 1.0 },
            Instruction::FunctionCall {
                name: String::from("func"),
                arg_count: 1
            },
        );
    }

    #[test]
    fn binary_function_call() {
        function_script_assert!(
            "func[1, 2]",
            Instruction::Number { value: 1.0 },
            Instruction::Number { value: 2.0 },
            Instruction::FunctionCall {
                name: String::from("func"),
                arg_count: 2
            },
        );
    }

    #[test]
    fn nary_function_call() {
        function_script_assert!(
            "func[1, 2, 3, 4, 5, 6]",
            Instruction::Number { value: 1.0 },
            Instruction::Number { value: 2.0 },
            Instruction::Number { value: 3.0 },
            Instruction::Number { value: 4.0 },
            Instruction::Number { value: 5.0 },
            Instruction::Number { value: 6.0 },
            Instruction::FunctionCall {
                name: String::from("func"),
                arg_count: 6
            },
        );
    }

    #[test]
    fn if_call() {
        function_script_assert!(
            "if[a, b, c]",
            Instruction::Variable {
                name: String::from("a")
            },
            Instruction::Variable {
                name: String::from("b")
            },
            Instruction::Variable {
                name: String::from("c")
            },
            Instruction::FunctionCall {
                name: String::from("if"),
                arg_count: 3
            },
        );
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
    fn basic_order_of_operations() {
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

    #[test]
    fn complex_order_of_operations() {
        function_script_assert!(
            "!-func[1] / 2 * 3 + 4 - 5 == 6 != 7 < 8 > 9 <= 10 >= 11 & 12 ^ 13 | 14",
            Instruction::Number { value: 1.0 },
            Instruction::FunctionCall {
                name: String::from("func"),
                arg_count: 1
            },
            Instruction::UnaryNegative,
            Instruction::Number { value: 2.0 },
            Instruction::Division,
            Instruction::Number { value: 3.0 },
            Instruction::Multiplication,
            Instruction::Number { value: 4.0 },
            Instruction::Addition,
            Instruction::Number { value: 5.0 },
            Instruction::Subtraction,
            Instruction::Number { value: 6.0 },
            Instruction::Equal,
            Instruction::Number { value: 7.0 },
            Instruction::NotEqual,
            Instruction::Number { value: 8.0 },
            Instruction::Less,
            Instruction::Number { value: 9.0 },
            Instruction::Greater,
            Instruction::Number { value: 10.0 },
            Instruction::LessEqual,
            Instruction::Number { value: 11.0 },
            Instruction::GreaterEqual,
            Instruction::UnaryLogicalNot,
            Instruction::Number { value: 12.0 },
            Instruction::LogicalAnd,
            Instruction::Number { value: 13.0 },
            Instruction::LogicalXor,
            Instruction::Number { value: 14.0 },
            Instruction::LogicalOr,
        );
    }

    #[test]
    fn decimal_numbers() {
        function_script_assert!("0.232", Instruction::Number { value: 0.232 },);
        function_script_assert!("0.", Instruction::Number { value: 0.0 },);
        function_script_assert!("0.1E2", Instruction::Number { value: 0.1E2 },);
    }

    #[test]
    fn integration_test() {
        function_script_assert!(
            "if[leftDoorsTarget == 0, if[leftDoors >= 0.496, 0.9368 * leftDoors - 0.3068, exp[4.684 * leftDoors - \
             4.05] - 0.02], if[leftDoors <= 0.5068, 0.9368 * leftDoors, -exp[-4.684 * leftDoors + 0.615] + 0.647]]",
            Instruction::Variable {
                name: String::from("leftDoorsTarget")
            },
            Instruction::Number { value: 0.0 },
            Instruction::Equal,
            Instruction::Variable {
                name: String::from("leftDoors")
            },
            Instruction::Number { value: 0.496 },
            Instruction::GreaterEqual,
            Instruction::Number { value: 0.9368 },
            Instruction::Variable {
                name: String::from("leftDoors")
            },
            Instruction::Multiplication,
            Instruction::Number { value: 0.3068 },
            Instruction::Subtraction,
            Instruction::Number { value: 4.684 },
            Instruction::Variable {
                name: String::from("leftDoors")
            },
            Instruction::Multiplication,
            Instruction::Number { value: 4.05 },
            Instruction::Subtraction,
            Instruction::FunctionCall {
                name: String::from("exp"),
                arg_count: 1
            },
            Instruction::Number { value: 0.02 },
            Instruction::Subtraction,
            Instruction::FunctionCall {
                name: String::from("if"),
                arg_count: 3
            },
            Instruction::Variable {
                name: String::from("leftDoors")
            },
            Instruction::Number { value: 0.5068 },
            Instruction::LessEqual,
            Instruction::Number { value: 0.9368 },
            Instruction::Variable {
                name: String::from("leftDoors")
            },
            Instruction::Multiplication,
            Instruction::Number { value: 4.684 },
            Instruction::UnaryNegative,
            Instruction::Variable {
                name: String::from("leftDoors")
            },
            Instruction::Multiplication,
            Instruction::Number { value: 0.615 },
            Instruction::Addition,
            Instruction::FunctionCall {
                name: String::from("exp"),
                arg_count: 1
            },
            Instruction::UnaryNegative,
            Instruction::Number { value: 0.647 },
            Instruction::Addition,
            Instruction::FunctionCall {
                name: String::from("if"),
                arg_count: 3
            },
            Instruction::FunctionCall {
                name: String::from("if"),
                arg_count: 3
            },
        )
    }
}
