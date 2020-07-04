use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag_no_case},
    character::complete::char as char_c,
    combinator::{map_res, opt},
    sequence::{delimited, preceded},
    IResult,
};
use std::future::Future;

pub struct FileInput<'a> {
    base_path: &'a str,
    requested_path: &'a str,
}

pub struct FileOutput {
    path: String,
    output: String,
}

pub struct PreprocessingContext<FileFunc, FileFuncFut>
where
    FileFunc: Fn(FileInput<'_>) -> FileFuncFut,
    FileFuncFut: Future<Output = FileOutput>,
{
    file_func: FileFunc,
}
impl<FileFunc, FileFuncFut> PreprocessingContext<FileFunc, FileFuncFut>
where
    FileFunc: Fn(FileInput<'_>) -> FileFuncFut,
    FileFuncFut: Future<Output = FileOutput>,
{
    pub fn new(file_func: FileFunc) -> Self {
        Self { file_func }
    }

    pub fn preprocess_file(&self, file_path: &str, file_contents: &str) -> Option<String> {
        let mut output = String::with_capacity(file_contents.len() * 2);
        let mut contents = file_contents;
        while !contents.is_empty() {
            let (remaining, out) = self.glob(contents).ok()?;
            output += out;

            let remaining = match self.directive(remaining) {
                Ok((remaining, out)) => {
                    output.push_str(&out);
                    remaining
                }
                Err(..) => {
                    let (remaining, _) = self.new_line(remaining).ok()?;
                    remaining
                }
            };

            contents = remaining;
        }
        unimplemented!()
    }

    fn value_or_directive<'i>(&self, input: &'i str) -> IResult<&'i str, i64> {
        alt((|i: &'i str| self.value(i), |i: &'i str| self.value_directive(i)))(input)
    }

    fn value<'i>(&self, input: &'i str) -> IResult<&'i str, i64> {
        map_res(is_a("0123456789"), |i: &'i str| i.parse())(input)
    }

    fn value_directive<'i>(&self, input: &'i str) -> IResult<&'i str, i64> {
        map_res(|i: &'i str| self.directive(i), |v| v.parse())(input)
    }

    fn directive<'i>(&self, input: &'i str) -> IResult<&'i str, String> {
        preceded(char_c('$'), alt((|i: &'i str| self.directive_chr(i),)))(input)
    }

    fn directive_chr<'i>(&self, input: &'i str) -> IResult<&'i str, String> {
        let (input, _) = tag_no_case("chr")(input)?;
        map_res(
            delimited(char_c('('), |i: &str| self.value_or_directive(i), char_c(')')),
            |value| match value {
                x @ 10 | x @ 13 | x @ 20..=127 => {
                    let mut string = String::new();
                    string.push(char::from(x as u8));
                    Ok(string)
                }
                _ => Err(()),
            },
        )(input)
    }

    fn glob<'i>(&self, input: &'i str) -> IResult<&'i str, &'i str> {
        is_not("$\r\n")(input)
    }

    fn new_line<'i>(&self, input: &'i str) -> IResult<&'i str, ()> {
        let (input, out) = opt(is_a("\r\n"))(input)?;
        if out.is_some() {
            let (input, _) = opt(|i| self.comment(i))(input)?;
            Ok((input, ()))
        } else {
            Ok((input, ()))
        }
    }

    fn comment<'i>(&self, input: &'i str) -> IResult<&'i str, ()> {
        let (input, _) = char_c(';')(input)?;
        let (input, _) = is_not("\r\n")(input)?;
        Ok((input, ()))
    }
}

#[cfg(test)]
mod test {
    use super::{FileInput, FileOutput, PreprocessingContext};
    use std::future::Future;

    type DummyReturnFuture = impl Future<Output = FileOutput>;
    fn create_dummy_context() -> PreprocessingContext<impl Fn(FileInput<'_>) -> DummyReturnFuture, DummyReturnFuture> {
        PreprocessingContext::new(|input| async move {
            FileOutput {
                output: String::new(),
                path: String::new(),
            }
        })
    }

    #[test]
    fn glob() {
        let ctx = create_dummy_context();
        assert_eq!(ctx.glob("hi, hello; how're you"), Ok(("", "hi, hello; how're you")));
        assert_eq!(ctx.glob("hi, hello;\nhow're you"), Ok(("\nhow're you", "hi, hello;")));
        assert_eq!(ctx.glob("hi, hello;$how're you"), Ok(("$how're you", "hi, hello;")));
    }

    #[test]
    fn comment() {
        let ctx = create_dummy_context();
        assert_eq!(ctx.comment(";I am a comment"), Ok(("", ())));
        assert_eq!(ctx.comment(";I am a comment\n"), Ok(("\n", ())));
        assert!(matches!(
            ctx.comment(" ;I am not a comment\n"),
            Err(nom::Err::Error((" ;I am not a comment\n", ..)))
        ));
    }

    #[test]
    fn new_line() {
        let ctx = create_dummy_context();
        assert_eq!(ctx.new_line("\n\r\n\r\n"), Ok(("", ())));
        assert_eq!(ctx.new_line("\n\r\n\r;comment\n"), Ok(("\n", ())));
        assert_eq!(ctx.new_line(";I am a comment"), Ok((";I am a comment", ())));
    }
}
