use std::borrow::Cow;

#[must_use]
#[allow(clippy::missing_const_for_fn)] // Needed for windows platforms
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
