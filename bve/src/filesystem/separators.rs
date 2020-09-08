use std::borrow::Cow;

pub fn convert_path_separators(input: &str) -> Cow<'_, str> {
    #[cfg(windows)]
    {
        Cow::Borrowed(input)
    }
    #[cfg(not(windows))]
    {
        Cow::Owned(input.replace('\\', "/"))
    }
}
