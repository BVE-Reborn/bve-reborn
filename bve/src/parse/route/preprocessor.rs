use nom::{
    bytes::complete::{is_a, is_not, take_while},
    character::complete::{char as char_c, none_of},
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
            let (remaining, output) = self.glob(contents).ok()?;
            contents += output;
        }
        unimplemented!()
    }

    fn glob(&self, input: &str) -> IResult<&str, &str> {
        is_not(";$")(input)
    }

    fn comment(&self, input: &str) -> IResult<&str, ()> {
        let (input, _) = char_c(';')(input)?;
        let (input, _) = none_of("\r\n")(input)?;
        let (input, _) = is_a("\r\n")(input)?;
        Ok((input, ()))
    }
}
