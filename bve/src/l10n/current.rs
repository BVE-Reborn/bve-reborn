use crate::l10n::Language;
use locale_config::LanguageRange;

#[must_use]
pub fn get_current_language() -> Language {
    let locale = locale_config::Locale::user_default();

    // Look for the message language
    if let Some((_category, range)) = locale.tags().find(|(c, _)| *c == Some("messages")) {
        parse_language_range(&range)
    } else {
        // If it's not there, just grab the first one.
        if let Some((_category, range)) = locale.tags().next() {
            parse_language_range(&range)
        } else {
            Language::EN
        }
    }
}

fn parse_language_range(range: &LanguageRange<'_>) -> Language {
    let range_str = range.to_string();

    if range_str.is_empty() {
        return Language::EN;
    }

    let locale = unic_locale::Locale::from_bytes(range_str.as_bytes()).expect("Unable to parse locale");

    Language::from_code(locale.langid.language())
}
